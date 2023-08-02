use std::convert::Infallible;

use crate::db::Table;

#[derive(jms_macros::DbPartialUpdate, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct JmsComponent {
  pub id: String,
  pub name: String,
  pub symbol: String,
  pub last_tick: chrono::DateTime<chrono::Local>
}

impl Table for JmsComponent {
  const PREFIX: &'static str = "components";
  type Id = String;
  type Err = Infallible;

  fn id(&self) -> String {
    self.id.clone()
  }
}