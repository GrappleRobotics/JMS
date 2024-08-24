use jms_core_lib::db::Singleton;

#[derive(jms_macros::Updateable)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TBASettings {
  pub auth_id: Option<String>,
  pub auth_key: Option<String>,
  pub api_key: Option<String>,
}

impl Default for TBASettings {
  fn default() -> Self {
    Self { auth_id: None, auth_key: None, api_key: Some("570EPOfXIRWFD38D6jkMPWOpd1gWfAeb02OORIqr8YRwxNhqeC83cKqTgsVIQz7U".to_owned()) }
  }
}

impl Singleton for TBASettings {
  const KEY: &'static str = "db:tba";
}

#[jms_macros::service]
pub trait TBARPC {
  async fn update_now() -> Result<(), String>;
}