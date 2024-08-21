use jms_core_lib::db::Singleton;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq, Eq)]
pub enum RadioType {
  Linksys,
  Unifi
}

#[derive(jms_macros::Updateable)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct NetworkingSettings {
  pub router_username: String,
  pub router_password: String,

  pub radio_type: RadioType,
  pub radio_username: String,
  pub radio_password: String,

  pub team_channel: Option<usize>,
  pub admin_channel: Option<usize>,
  pub admin_ssid: Option<String>,
  pub admin_password: Option<String>,

  pub guest_ssid: Option<String>,
  pub guest_password: Option<String>
}

impl Default for NetworkingSettings {
  fn default() -> Self {
    Self {
      router_username: "admin".to_owned(),
      router_password: "jmsR0cks".to_owned(),
      
      radio_type: RadioType::Unifi,
      radio_username: "FTA".to_owned(),
      radio_password: "jmsR0cks".to_owned(),

      team_channel: None,
      admin_channel: None,
      admin_ssid: Some("JMS".to_owned()),
      admin_password: Some("myEventR0cks".to_owned()),

      guest_ssid: Some("Team WiFi".to_owned()),
      guest_password: None,
    }
  }
}

impl Singleton for NetworkingSettings {
  const KEY: &'static str = "db:networking";
}

#[jms_macros::service]
pub trait JMSNetworkingRPC {
  async fn configure_admin() -> Result<(), String>;
  async fn force_reprovision() -> Result<(), String>;
}