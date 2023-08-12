use std::{net::{Ipv4Addr, SocketAddr}, time::Duration};

use jms_networking_lib::NetworkingSettings;
use log::info;

use crate::{NetworkConfig, ssh::{SSHSession, CommandResult}};

pub async fn configure_ap_teams(config: &NetworkConfig, settings: &NetworkingSettings) -> anyhow::Result<()> {
  info!("Configuring Radio (Teams)...");
  let addr = SocketAddr::new(Ipv4Addr::new(10, 0, 100, 2).into(), 22);
  let session = SSHSession::connect(addr, &settings.radio_username, &settings.radio_password).await?;

  let mut cfgs = vec![];

  for (i, (team, wpa_key)) in vec![ &config.blue1, &config.blue2, &config.blue3, &config.red1, &config.red2, &config.red3 ].into_iter().enumerate() {
    let iface_num = i + 1;
    match wpa_key {
      Some(wpa_key) => {
        cfgs.push(format!("set wireless.@wifi-iface[{}].disabled='0'", iface_num));
        cfgs.push(format!("set wireless.@wifi-iface[{}].ssid='{}'", iface_num, team));
        cfgs.push(format!("set wireless.@wifi-iface[{}].key='{}'", iface_num, wpa_key));
        cfgs.push(format!("set wireless.@wifi-iface[{}].encryption='psk2'", iface_num));
      },
      None => {
        info!("Station {} unoccupied", iface_num);
        cfgs.push(format!("set wireless.@wifi-iface[{}].disabled='1'", iface_num));
        cfgs.push(format!("set wireless.@wifi-iface[{}].ssid='{}'", iface_num, team));
        cfgs.push(format!("set wireless.@wifi-iface[{}].key='unoccupied'", iface_num));
        cfgs.push(format!("set wireless.@wifi-iface[{}].encryption='psk2'", iface_num));
      }
    }
  }

  cfgs.push("set wireless.radio0.disabled='0'".to_owned());
  cfgs.push("commit wireless".to_owned());

  let chan_cfg = format!(
    "set wireless.radio0.channel='{}'",
    settings
      .team_channel
      .map_or("auto".to_owned(), |c| format!("{}", c))
  );
  let mut cfgs: Vec<&str> = cfgs.iter().map(|x| x.as_str()).collect();
  cfgs.insert(0, chan_cfg.as_str());

  do_uci(&session, false, &cfgs).await?;

  info!("Radio Configured!");
  Ok(())
}

pub async fn configure_ap_admin(settings: &NetworkingSettings) -> anyhow::Result<()> {
  info!("Configuring Radio (Admin)...");
  let addr = SocketAddr::new(Ipv4Addr::new(10, 0, 100, 2).into(), 22);
  let session = SSHSession::connect(addr, &settings.radio_username, &settings.radio_password).await?;

  do_uci(
    &session,
    true,
    &vec![
      format!(
        "set wireless.radio1.channel='{}'",
        settings
          .admin_channel
          .map_or("auto".to_owned(), |c| format!("{}", c))
      )
      .as_str(),
      format!(
        "set wireless.radio1.disabled='{}'",
        settings.admin_ssid.is_none() as usize
      )
      .as_str(),
      format!(
        "set wireless.@wifi-iface[0].ssid='{}'",
        settings.admin_ssid.as_ref().unwrap_or(&"no-admin".to_owned())
      )
      .as_str(),
      format!(
        "set wireless.@wifi-iface[0].key='{}'",
        settings.admin_password.as_ref().unwrap_or(&"".to_owned())
      )
      .as_str(),
      "set wireless.@wifi-iface[0].encryption='psk2'",
      "commit wireless",
    ],
  )
  .await?;

  info!("Radio Configured!");
  Ok(())
}

async fn do_uci(session: &SSHSession, admin: bool, cmds: &[&str]) -> anyhow::Result<CommandResult> {
  let cmd = format!("uci batch <<EOI && {}\nEOI\n", cmds.join("\n"));
  let reply = session.run(&cmd).await?;
  if !reply.success() {
    anyhow::bail!("Failed to apply UCI {}", reply.output());
  }

  tokio::time::sleep(Duration::from_millis(1000)).await;

  if admin {
    let reply = session.run("/sbin/wifi down radio1; /sbin/wifi up radio1").await?;
    if !reply.success() {
      anyhow::bail!("Failed to reload radio1 {}", reply.output());
    }
  } else {
    let reply = session.run("/sbin/wifi down radio0; /sbin/wifi up radio0").await?;
    if !reply.success() {
      anyhow::bail!("Failed to reload radio0 {}", reply.output());
    }
  }

  Ok(reply)
}