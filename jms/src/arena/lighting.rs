use std::{collections::HashMap, str::FromStr};

use strum::IntoEnumIterator;

use crate::{config::Inquirable, models};

use super::station::AllianceStationId;

pub type Colour = palette::Srgb<u8>;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum LightMode {
  Off,
  Constant(Colour),
  Pulse(Colour, chrono::Duration),
  Chase(Colour, chrono::Duration),
  Rainbow(chrono::Duration)
}

#[derive(Debug, Clone)]
pub struct ArenaLighting {
  pub teams: HashMap<models::Alliance, Vec<LightMode>>,
  pub scoring_table: HashMap<models::Alliance, LightMode>,
  pub settings: ArenaLightingSettings
}

impl ArenaLighting {
  pub fn new(settings: ArenaLightingSettings) -> Self {
    let mut al = Self {
      teams: HashMap::new(),
      scoring_table: HashMap::new(),
      settings
    };

    for t in models::Alliance::iter() {
      al.teams.insert(t, vec![LightMode::Off, LightMode::Off, LightMode::Off]);
      al.scoring_table.insert(t, LightMode::Off);
    }

    al
  }

  pub fn set_team(&mut self, station: AllianceStationId, mode: LightMode) {
    let teams = self.teams.get_mut(&station.alliance);
    if let Some(teams) = teams {
      teams[(station.station - 1) as usize] = mode;
    } else {
      let mut v = vec![LightMode::Off, LightMode::Off, LightMode::Off];
      v[(station.station - 1) as usize] = mode;
      self.teams.insert(station.alliance, v);
    }
  }

  pub fn set_table(&mut self, alliance: models::Alliance, mode: LightMode) {
    self.scoring_table.insert(alliance, mode);
  }

  pub fn set_all(&mut self, mode: LightMode) {
    self.set_alliance(models::Alliance::Blue, mode);
    self.set_alliance(models::Alliance::Red, mode);
  }

  pub fn set_alliance(&mut self, alliance: models::Alliance, mode: LightMode) {
    for t in self.teams.get_mut(&alliance).unwrap().iter_mut() {
      *t = mode;
    }

    self.scoring_table.insert(alliance, mode);
  }
}


#[derive(Debug, Clone)]
pub struct ArenaLightingSettings {
  pub red: Colour,
  pub blue: Colour,
  pub idle: LightMode,
  pub team_estop: LightMode,
  pub field_estop: LightMode,
  pub field_reset: LightMode,
  pub field_reset_teams: LightMode,
  pub match_armed_red: LightMode,
  pub match_armed_blue: LightMode,
}

impl Default for ArenaLightingSettings {
  fn default() -> Self {
    Self { 
      red: Colour::new(255, 0, 0), 
      blue: Colour::new(0, 0, 255), 
      idle: LightMode::Off,
      team_estop: LightMode::Pulse(
        Colour::new(255, 145, 0),
        chrono::Duration::seconds(1)
      ),
      field_estop: LightMode::Constant(
        Colour::new(255, 145, 0)
      ),
      field_reset: LightMode::Constant(
        Colour::new(0, 0, 255)
      ),
      field_reset_teams: LightMode::Constant(
        Colour::new(255, 0, 255)
      ),
      match_armed_red: LightMode::Pulse(
        Colour::new(255, 0, 0),
        chrono::Duration::seconds(1)
      ),
      match_armed_blue: LightMode::Pulse(
        Colour::new(255, 0, 0),
        chrono::Duration::seconds(1)
      )
    }
  }
}

impl Inquirable for Colour {
  fn inquire(msg: &'static str, default: Option<&Self>) -> inquire::CustomType<'static, Self> {
    let default = match default {
      Some(&d) => Some((d, (&|v| format!("{:x}", v)) as &dyn Fn(Self) -> String)),
      None => None,
    };

    inquire::CustomType {
      message: msg,
      default,
      help_message: None,
      formatter: &|v| format!("{:x}", v),
      parser: &|i| Colour::from_str(i).or(Err(())),
      error_message: "Must be in hex code format".into(),
    }
  }
}

// #[async_trait::async_trait]
// impl Interactive for ArenaLightingSettings {
//   async fn interactive() -> anyhow::Result<Self> {
//     let mut als = ArenaLightingSettings::default();
//     let configure = inquire::Confirm::new("Do you want to customise field lighting?").prompt()?;
//     if configure {
//       als.red = Colour::inquire("Red Alliance Colour", Some(&als.red)).prompt()?;
//       als.blue = Colour::inquire("Blue Alliance Colour", Some(&als.blue)).prompt()?;
//     }
//     Ok(als)
//   }
// }