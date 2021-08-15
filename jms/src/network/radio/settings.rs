use std::net::Ipv4Addr;

use crate::config::Interactive;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FieldRadioSettings {
  pub ip: Ipv4Addr,
  pub user: String,
  pub pass: String,

  // None = no admin
  pub admin_ssid: Option<String>,
  pub admin_key: Option<String>,
  
  // None = "auto"
  pub admin_channel: Option<usize>,
  pub team_channel: Option<usize>
}

#[async_trait::async_trait]
impl Interactive for FieldRadioSettings {
  async fn interactive() -> anyhow::Result<Self> {
    let ip = inquire::CustomType::<Ipv4Addr>::new("Field Router IP?")
      .with_error_message("Must be a valid IPv4 Address")
      .with_default((Ipv4Addr::new(10, 0, 100, 2), &|ip| ip.to_string()))
      .prompt()?;

    let user = inquire::Text::new("Field Router Username?").with_default("root").prompt()?;
    let password = inquire::Text::new("Field Router Password?").with_default("root").prompt()?;

    let mut cfg = FieldRadioSettings {
      ip,
      user,
      pass: password,
      admin_ssid: None,
      admin_key: None,
      admin_channel: None,
      team_channel: None
    };

    if inquire::Confirm::new("Do you want to setup an admin network?").with_default(true).prompt()? {
      cfg.admin_ssid = Some(inquire::Text::new("Admin SSID?").with_default("JMS").prompt()?);
      cfg.admin_key = Some(inquire::Text::new("Admin Password?").with_default("jmsR0cks").prompt()?);
    }

    if inquire::Confirm::new("Do you want to manually configure radio channels?").with_default(false).prompt()? {
      cfg.team_channel = Some(inquire::CustomType::<usize>::new("Team Channel?").prompt()?);
      if cfg.admin_ssid.is_some() {
        cfg.admin_channel = Some(inquire::CustomType::<usize>::new("Admin Channel?").prompt()?);
      }
    }

    Ok(cfg)
  }
}