use jms_core_lib::db::Singleton;

#[derive(jms_macros::Updateable)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct NetworkingSettings {
  pub router_username: String,
  pub router_password: String,
  pub radio_username: String,
  pub radio_password: String,

  pub team_channel: Option<usize>,
  pub admin_channel: Option<usize>,
  pub admin_ssid: Option<String>,
  pub admin_password: Option<String>
}

impl Default for NetworkingSettings {
  fn default() -> Self {
    Self {
      router_username: "admin".to_owned(),
      router_password: "jmsR0cks".to_owned(),
      radio_username: "root".to_owned(),
      radio_password: "root".to_owned(),

      team_channel: None,
      admin_channel: None,
      admin_ssid: Some("JMS".to_owned()),
      admin_password: Some("myEventR0cks".to_owned())
    }
  }
}

impl Singleton for NetworkingSettings {
  const KEY: &'static str = "db:networking";
}

#[jms_macros::service]
pub trait JMSNetworkingRPC {
  async fn configure_admin() -> Result<(), String>;
}