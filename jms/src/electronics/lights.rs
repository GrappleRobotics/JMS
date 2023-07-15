use chrono::{DateTime, Local, Duration};
use palette::{Hsv, Srgb, FromColor};

use crate::{arena::{Arena, station::AllianceStationId, state::ArenaState, matches::MatchPlayState}, models::Alliance};

use super::settings::LightingConfig;

pub type Colour = palette::Srgb<u8>;

#[derive(Debug, Clone)]
pub enum ModuleLightingMode {
  Off,
  Solid(Colour),
  Flash { colour: Colour, period: chrono::Duration, offset: f64, duty: f64 },
  Breathe { colour: Colour, period: chrono::Duration, offset: f64 },
  Rainbow { period: chrono::Duration, offset: f64 }
}

impl ModuleLightingMode {
  fn calc_fraction(time: &DateTime<Local>, period: &chrono::Duration, offset: f64) -> f64 {
    let now_ms = time.timestamp_millis();
    let period_ms = period.num_milliseconds();
    ((now_ms % period_ms) as f64 / (period_ms as f64) + offset) % 1.0
  }

  pub fn calculate(&self, time: &DateTime<Local>) -> Colour {
    match &self {
      ModuleLightingMode::Off => Colour::new(0, 0, 0),
      ModuleLightingMode::Solid(colour) => colour.clone(),
      ModuleLightingMode::Flash { colour, period, offset, duty } => {
        let fraction = Self::calc_fraction(time, &period, *offset);
        if fraction <= *duty { colour.clone() }
        else { Colour::new(0, 0, 0) }
      },
      ModuleLightingMode::Breathe { colour, period, offset } => {
        let fraction = Self::calc_fraction(time, &period, *offset);
        let scale = if fraction <= 0.5 {
          fraction * 2.0
        } else {
          1.0 - (fraction - 0.5) * 2.0
        };
        Colour::new(
          (colour.red as f64 * scale) as u8,
          (colour.green as f64 * scale) as u8,
          (colour.blue as f64 * scale) as u8,
        )
      },
      ModuleLightingMode::Rainbow { period, offset } => {
        let fraction = Self::calc_fraction(time, &period, *offset);
        let colour_hsv = Hsv::new::<f64>(fraction, 1.0, 1.0);
        let rgb = Srgb::from_color(colour_hsv);
        Colour::new(
          (rgb.red * 255.0) as u8,
          (rgb.green * 255.0) as u8,
          (rgb.blue * 255.0) as u8,
        )
      },
    }
  }
}

pub type DMXData = [u8; 512];

macro_rules! dmx_update {
  ($dmx:expr, $config:expr, $values:expr, $time:expr) => {
    for i in 0..$values.len() {
      let base_addr = $config[i].base_address - 1;
      for module in 0..$config[i].n_modules {
        let v = $values[i][module].calculate($time);
        $dmx[base_addr+module*3] = v.red;
        $dmx[base_addr+module*3+1] = v.green;
        $dmx[base_addr+module*3+2] = v.blue;
      }
    }
  }
}

pub struct FieldLights {
  config: LightingConfig,
  blue: [ Vec<ModuleLightingMode>; 3 ],
  red: [ Vec<ModuleLightingMode>; 3 ],
  scoring_table: [ Vec<ModuleLightingMode>; 2 ],
}

impl FieldLights {
  pub fn new(config: LightingConfig) -> Self {
    Self {
      blue: [
        vec![ModuleLightingMode::Off; config.blue[0].n_modules],
        vec![ModuleLightingMode::Off; config.blue[1].n_modules],
        vec![ModuleLightingMode::Off; config.blue[2].n_modules],
      ],
      red: [
        vec![ModuleLightingMode::Off; config.red[0].n_modules],
        vec![ModuleLightingMode::Off; config.red[1].n_modules],
        vec![ModuleLightingMode::Off; config.red[2].n_modules],
      ],
      scoring_table: [
        vec![ModuleLightingMode::Off; config.scoring_table[0].n_modules],
        vec![ModuleLightingMode::Off; config.scoring_table[1].n_modules],
      ],
      config,
    }
  }

  async fn update_modes(&mut self, arena: Arena) -> anyhow::Result<()> {
    // Zero out everything
    let els = self.blue.iter_mut().chain(self.red.iter_mut()).chain(self.scoring_table.iter_mut());
    els.flat_map(|x| x.iter_mut()).for_each(|v| {
      *v = ModuleLightingMode::Off
    });

    let mut let_state_based = false;

    match arena.state().await {
      ArenaState::Init | ArenaState::Reset => { },
      ArenaState::Idle { .. } => { },
      ArenaState::Estop => {
        // Set lights to alternate red  / black
        let els = self.blue.iter_mut().chain(self.red.iter_mut()).chain(self.scoring_table.iter_mut());
        els.flat_map(|x| x.iter_mut()).enumerate().for_each(|(i, v)| {
          *v = ModuleLightingMode::Flash { colour: Colour::new(255, 0, 0), period: Duration::seconds(1), offset: (i % 2) as f64 * 0.5, duty: 0.5 }
        });
      },
      ArenaState::Prestart { net_ready } => {
        let_state_based = true;
        if !net_ready {
          // Set scoring table lights to orange
          for el in self.scoring_table.as_mut() {
            el.iter_mut().for_each(|v| *v = ModuleLightingMode::Solid(Colour::new(252, 64, 3)))
          }
        }
      },
      ArenaState::MatchArmed => {
        let_state_based = true;
        // Set scoring table lights to pulsing yellow
        for el in self.scoring_table.as_mut() {
          el.iter_mut().for_each(|v| {
            *v = ModuleLightingMode::Breathe { colour: Colour::new(252, 223, 3), period: Duration::seconds(4), offset: 0.0 }
          })
        }
      },
      ArenaState::MatchPlay => {
        let_state_based = true;
        // Depending on the match state, set the alliance colours
        if let Some(m) = arena.current_match().await {
          match m.state {
            MatchPlayState::Warmup => {
              // Set all lights to white to get the teams on their feet
              let els = self.blue.iter_mut().chain(self.red.iter_mut()).chain(self.scoring_table.iter_mut());
              els.flat_map(|x| x.iter_mut()).for_each(|v| {
                *v = ModuleLightingMode::Solid(Colour::new(255, 255, 255));
              });
            },
            MatchPlayState::Auto | MatchPlayState::Teleop => {
              // Alliance Colours
              let blue = self.blue.iter_mut().chain(self.scoring_table.iter_mut().take(1));
              for el in blue.flat_map(|x| x.iter_mut()) {
                *el = ModuleLightingMode::Solid(Colour::new(0, 0, 255));
              }
              
              let red = self.red.iter_mut().chain(self.scoring_table.iter_mut().skip(1).take(1));
              for el in red.flat_map(|x| x.iter_mut()) {
                *el = ModuleLightingMode::Solid(Colour::new(255, 0, 0));
              }
            },
            _ => (),
          }
        }
      },
      ArenaState::MatchComplete { .. } => { },
    };

    // Update station modules based on station state
    if let_state_based {
      for (alliance, iter) in [(Alliance::Blue, self.blue.iter_mut()), (Alliance::Red, self.red.iter_mut())] {
        let colour = if alliance == Alliance::Blue { Colour::new(0, 0, 255) } else { Colour::new(255, 0, 0) };
        let mut ok = true;

        for (i, els) in iter.enumerate() {
          if let Some(stn) = arena.station_for_id(AllianceStationId { alliance, station: (i + 1) as u32 }).await {
            // Pick a mode based on the state of the station
            let mode = {
              let state = stn.read().await;
              if state.estop || state.astop {
                // Estop - solid orange
                Some((
                  ModuleLightingMode::Solid(Colour::new(255, 64, 0)),
                  ModuleLightingMode::Solid(Colour::new(255, 64, 0)),
                ))
              } else if state.bypass {
                None
              } else if !state.connection_ok() {
                // Flash orange
                ok = false;
                Some((
                  ModuleLightingMode::Flash { colour: Colour::new(255, 64, 0), period: Duration::seconds(2), offset: 0.5, duty: 0.5 },
                  ModuleLightingMode::Flash { colour: Colour::new(255, 64, 0), period: Duration::seconds(2), offset: 0.0, duty: 0.5 },
                ))
              } else {
                None
              }
            };

            // Set the mode on the 1st and 3rd thirds of the module (or the whole thing if <= 2 modules)
            if let Some(mode) = mode {
              let n = (els.len() as f64 / 3.0).ceil() as usize;
              for j in 0..els.len() {
                if j < n { els[j] = mode.0.clone() }
                if (els.len() - n) < j { els[j] = mode.1.clone() }
              }
            }
          }
        
          // Flash the last element of the scoring table orange if there's a team in trouble
          if !ok {
            if alliance == Alliance::Blue {
              self.scoring_table[0][0] = ModuleLightingMode::Flash { colour: Colour::new(255, 64, 0), period: Duration::seconds(2), offset: 0.0, duty: 0.5 }
            } else {
              let len = els.len();
              self.scoring_table[1][len- 1] = ModuleLightingMode::Flash { colour: Colour::new(255, 64, 0), period: Duration::seconds(2), offset: 0.0, duty: 0.5 }
            }
          }
        }
      }
    }

    Ok(())
  }

  pub async fn update(&mut self, arena: Arena) -> anyhow::Result<DMXData> {
    let mut dmx = [0u8; 512];

    let now = chrono::Local::now();

    self.update_modes(arena).await?;

    dmx_update!(dmx, self.config.blue, self.blue, &now);
    dmx_update!(dmx, self.config.red, self.red, &now);
    dmx_update!(dmx, self.config.scoring_table, self.scoring_table, &now);

    Ok(dmx)
  }
}