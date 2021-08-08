use std::net::{Ipv4Addr, SocketAddr};

use anyhow::{bail, Result};

use crate::{arena::station::AllianceStationId, utils::ssh::{CommandResult, SSHSession}};

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

impl Default for FieldRadioSettings {
  fn default() -> Self {
    Self {
      ip: Ipv4Addr::new(10, 0, 100, 2),
      user: "root".to_owned(),
      pass: "root".to_owned(),
      admin_ssid: Some("JMS".to_owned()),
      admin_key: Some("jmsR0cks".to_owned()),
      admin_channel: None,
      team_channel: None
    }
  }
}

pub struct TeamRadioConfig {
  pub station: AllianceStationId,
  pub team: Option<usize>,
  pub wpakey: Option<String>,
}

pub struct FieldRadio {
  pub settings: FieldRadioSettings
}

impl FieldRadio {
  pub fn new(settings: FieldRadioSettings) -> Self {
    Self { settings }
  }

  pub async fn configure(&self, teams: &[TeamRadioConfig]) -> Result<()> {
    info!("Configuring Radio...");
    let addr = SocketAddr::new(self.settings.ip.into(), 22);
    let session = SSHSession::connect(addr, &self.settings.user, &self.settings.pass).await?;

    self.configure_admin(&session).await?;
    self.configure_teams(&session, teams).await?;
    info!("Radio Configured!");
    Ok(())
  }

  async fn configure_admin(&self, session: &SSHSession) -> Result<()> {
    self.do_uci(session, "wifi radio1", &vec![
      format!("set wireless.radio1.channel='{}'", self.settings.admin_channel.map_or("auto".to_owned(), |c| format!("{}", c))).as_str(),
      format!("set wireless.radio1.disabled='{}'", self.settings.admin_ssid.is_none() as usize).as_str(),
      format!("set wireless.@wifi-iface[0].ssid='{}'", self.settings.admin_ssid.as_ref().unwrap_or(&"no-admin".to_owned())).as_str(),
      format!("set wireless.@wifi-iface[0].key='{}'", self.settings.admin_key.as_ref().unwrap_or(&"".to_owned())).as_str(),
      "commit wireless"
    ]).await?;
    Ok(())
  }

  async fn configure_teams(&self, session: &SSHSession, teams: &[TeamRadioConfig]) -> Result<()> {
    let mut cfgs: Vec<String> = teams.iter().flat_map(|x| {
      let radio_num = 1 + x.station.to_station_idx();

      match (x.team, x.wpakey.as_ref()) {
        (Some(team), Some(wpakey)) if wpakey.len() > 8 && wpakey.len() < 60 => {
          vec![
            format!("set wireless.@wifi-iface[{}].disabled='0'", radio_num),
            format!("set wireless.@wifi-iface[{}].ssid='{}'", radio_num, team),
            format!("set wireless.@wifi-iface[{}].key='{}'", radio_num, wpakey),
          ]
        },
        (Some(team), _) => {
          error!("Team {} does not have a valid WPA Key! Disabling...", team);
          vec![
            format!("set wireless.@wifi-iface[{}].disabled='1'", radio_num),
            format!("set wireless.@wifi-iface[{}].ssid='{}-no-key'", radio_num, team),
            format!("set wireless.@wifi-iface[{}].key='{}-no-key'", radio_num, team),
          ]
        },
        (None, _) => {
          vec![
            format!("set wireless.@wifi-iface[{}].disabled='1'", radio_num),
            format!("set wireless.@wifi-iface[{}].ssid='unoccupied-{}'", radio_num, radio_num),
            format!("set wireless.@wifi-iface[{}].key='unoccupied-{}'", radio_num, radio_num),
          ]
        }
      }
    }).collect();
    cfgs.push("commit wireless".to_owned());

    let chan_cfg = format!("set wireless.radio0.channel='{}'", self.settings.team_channel.map_or("auto".to_owned(), |c| format!("{}", c)));
    let mut cfgs: Vec<&str> = cfgs.iter().map(|x| x.as_str()).collect();
    cfgs.insert(0, chan_cfg.as_str());

    self.do_uci(session, "wifi radio0", &cfgs).await?;
    Ok(())
  }

  async fn do_uci(&self, session: &SSHSession, target: &str, cmds: &[&str]) -> Result<CommandResult> {
    let cmd = format!("uci batch <<EOI && {}\n{}\nEOI\n", target, cmds.join("\n"));
    let reply = session.run(&cmd).await?;
    if !reply.success() {
      bail!("Failed to apply UCI {} - {}", target, reply.output())
    }
    Ok(reply)
  }


}