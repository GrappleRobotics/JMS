#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FieldRadioSettings {
  pub ip: String,
  pub user: String,
  pub pass: String,

  pub admin_ssid: String,
  pub admin_key: String,
  
  // None = "auto"
  pub admin_channel: Option<usize>,
  pub team_channel: Option<usize>
}

impl Default for FieldRadioSettings {
  fn default() -> Self {
    Self {
      ip: "10.0.100.2".to_owned(),
      user: "root".to_owned(),
      pass: "root".to_owned(),
      admin_ssid: "JMS".to_owned(),
      admin_key: "jmsR0cks".to_owned(),
      admin_channel: None,
      team_channel: None
    }
  }
}

pub struct FieldRadio {
  pub settings: FieldRadioSettings
}

impl FieldRadio {
  pub fn new(settings: FieldRadioSettings) -> Self {
    Self { settings }
  }

  pub async fn run(&self) {
    
  }
}