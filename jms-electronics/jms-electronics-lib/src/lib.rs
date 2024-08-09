use std::convert::Infallible;

use grapple_frc_msgs::grapple::jms::{JMSElectronicsStatus, JMSRole};
use jms_core_lib::db::{Singleton, Table};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct FieldElectronicsEndpoint {
  pub mac: String,
  pub status: JMSElectronicsStatus,
}

impl Table for FieldElectronicsEndpoint {
  const PREFIX: &'static str = "db:electronics_endpoints";
  type Id = String;
  type Err = Infallible;

  fn id(&self) -> Self::Id {
    self.mac.clone()
  }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum FieldElectronicsUpdate {
  SetRole {
    mac: String,
    role: JMSRole
  },
  Blink {
    mac: String
  }
}

#[jms_macros::service]
pub trait FieldElectronicsServiceRPC {
  async fn update(update: FieldElectronicsUpdate) -> Result<(), String>;
  async fn reset_estops() -> Result<(), String>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq, Eq)]
pub enum EstopMode {
  NormallyOpen,
  NormallyClosed
}

#[derive(jms_macros::Updateable)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct FieldElectronicsSettings {
  pub estop_mode: EstopMode
}

impl Default for FieldElectronicsSettings {
  fn default() -> Self {
    Self { estop_mode: EstopMode::NormallyClosed }
  }
}

impl Singleton for FieldElectronicsSettings {
  const KEY: &'static str = "db:electronics";
}
