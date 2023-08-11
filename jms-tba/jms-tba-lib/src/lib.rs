use jms_core_lib::db::{Singleton, DBDuration};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TBASettings {
  pub auth_id: Option<String>,
  pub auth_key: Option<String>,
}

impl Default for TBASettings {
  fn default() -> Self {
    Self { auth_id: None, auth_key: None }
  }
}

impl Singleton for TBASettings {
  const KEY: &'static str = "db:tba";
}