pub mod comms;

use crate::models;

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
