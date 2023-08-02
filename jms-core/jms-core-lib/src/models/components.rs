use std::convert::Infallible;

use jms_base::kv;

use crate::db::Table;

#[derive(jms_macros::DbPartialUpdate, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct JmsComponent {
  pub id: String,
  pub name: String,
  pub symbol: String,
  pub last_tick: chrono::DateTime<chrono::Local>,
  pub timeout_ms: usize,
}

impl Table for JmsComponent {
  const PREFIX: &'static str = "components";
  type Id = String;
  type Err = Infallible;

  fn id(&self) -> String {
    self.id.clone()
  }
}

impl JmsComponent {
  pub fn new(id: &str, name: &str, symbol: &str, timeout_ms: usize) -> Self {
    Self {
      id: id.to_owned(),
      name: name.to_owned(),
      symbol: symbol.to_owned(),
      last_tick: chrono::Local::now(),
      timeout_ms
    }
  }

  pub fn tick(&mut self, kv: &kv::KVConnection) -> anyhow::Result<()> {
    self.set_last_tick(chrono::Local::now(), &kv)
  }

  pub fn heartbeat_ok(&self) -> bool {
    (chrono::Local::now() - self.last_tick) < chrono::Duration::milliseconds(self.timeout_ms as i64)
  }

  pub fn heartbeat_ok_for(id: &str, kv: &kv::KVConnection) -> anyhow::Result<bool> {
    Ok((chrono::Local::now() - Self::get_last_tick(id.to_owned(), kv)?) < chrono::Duration::milliseconds(Self::get_timeout_ms(id.to_owned(), kv)? as i64))
  }
}