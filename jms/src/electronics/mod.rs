pub mod comms;

use palette::Pixel;

use crate::{arena, models};

pub mod protos {
  include!(concat!(env!("OUT_DIR"), "/jms.electronics.rs"));
}

impl From<protos::NodeRole> for Option<models::Alliance> {
  fn from(nr: protos::NodeRole) -> Self {
    match nr {
      protos::NodeRole::NodeBlue => Some(models::Alliance::Blue),
      protos::NodeRole::NodeRed => Some(models::Alliance::Red),
      _ => None
    }
  }
}

impl From<arena::lighting::LightMode> for protos::Lights {
  fn from(arena_lights: arena::lighting::LightMode) -> Self {
    let lm = match arena_lights {
      arena::lighting::LightMode::Off => protos::lights::Mode::Off(
        protos::lights::Off { off: true }
      ),
      arena::lighting::LightMode::Constant(col) => {
        let p: [u8; 3] = col.into_raw();
        protos::lights::Mode::Constant(
          protos::lights::Constant { rgb: p.into() }
        )
      },
      arena::lighting::LightMode::Pulse(col, dur) => {
        let p: [u8; 3] = col.into_raw();
        let ms = dur.num_milliseconds();
        protos::lights::Mode::Pulse(
          protos::lights::Pulse { rgb: p.into(), speed: ms as u32 }
        )
      },
      arena::lighting::LightMode::Chase(col, dur) => {
        let p: [u8; 3] = col.into_raw();
        let ms = dur.num_milliseconds();
        protos::lights::Mode::Chase(
          protos::lights::Chase { rgb: p.into(), speed: ms as u32 }
        )
      },
      arena::lighting::LightMode::Rainbow(dur) => {
        let ms = dur.num_milliseconds();
        protos::lights::Mode::Rainbow(
          protos::lights::Rainbow { speed: ms as u32 }
        )
      },
    };

    Self { mode: Some(lm) }
  }
}