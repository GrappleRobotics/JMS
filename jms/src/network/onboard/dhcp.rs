use std::{fs::{File, OpenOptions}, io::{Read, Write}, net::Ipv4Addr, time::Duration};

use anyhow::bail;
use dbus::nonblock::Proxy;
use handlebars::Handlebars;
use ipnetwork::Ipv4Network;
use log::{error, info, warn};
use serde_json::json;
use serde::Serialize;

use crate::{arena::station::AllianceStationId, log_expect, network::NetworkResult, utils::{danger::danger_or_err, service_configs::ServiceConfigs, templates}};

const DHCP_FILE: &'static str = "/etc/dhcp/jms-dhcp.conf";
const DHCP_MASTER_CONF_FILE: &'static str = "/etc/dhcp/dhcpd.conf";

#[derive(Serialize)]
pub struct DHCPConfig {
  pub router: Ipv4Network,
  pub dhcp: (Ipv4Addr, Ipv4Addr),
}

#[derive(Serialize)]
pub struct TeamDHCPConfig {
  pub station: AllianceStationId,
  pub team: Option<u16>,
  pub cfg: Option<DHCPConfig>
}

pub async fn configure_dhcp(admin_cfg: DHCPConfig, stn_cfgs: &[TeamDHCPConfig]) -> NetworkResult<()> {
  log_expect!(danger_or_err());
  let mut file = File::create(DHCP_FILE)?;

  info!("Generating DHCP file...");
  generate_dhcp_conf(&mut file, &admin_cfg, stn_cfgs).await?;

  info!("Reloading DHCP service...");
  reload_dhcp_service().await?;

  info!("DHCP Ready!");
  Ok(())
}

async fn generate_dhcp_conf(file: &mut File, admin_cfg: &DHCPConfig, stn_cfgs: &[TeamDHCPConfig]) -> NetworkResult<()> {
  log_expect!(danger_or_err());

  match ServiceConfigs::get("match.dhcp.conf") {
    Some(match_data) => {
      let template_str = std::str::from_utf8(match_data.as_ref())?;
      let mut hbars = Handlebars::new();
      templates::setup(&mut hbars);

      let result = hbars.render_template(
        template_str,
        &json!({
          "admin": admin_cfg,
          "stations": stn_cfgs
        })
      )?;

      file.write_all(result.as_bytes())?;
    },
    None => {
      error!("No ServiceConfig exists: match.dhcp.conf");
      bail!(DHCPError { msg: "No ServiceConfig exists: match.dhcp.conf".to_owned() });
    },
  }

  Ok(())
}

async fn reload_dhcp_service() -> NetworkResult<()> {
  let (resource, conn) = dbus_tokio::connection::new_system_sync()?;
  tokio::spawn(async {
    resource.await;
  });

  let proxy = Proxy::new("org.freedesktop.systemd1", "/org/freedesktop/systemd1", Duration::from_secs(5), conn);

  // Reference: https://www.freedesktop.org/software/systemd/man/org.freedesktop.systemd1.html
  proxy.method_call("org.freedesktop.systemd1.Manager", "StopUnit", ("isc-dhcp-server.service", "replace")).await?;
  maybe_update_dhcpd_conf().await?;  
  proxy.method_call("org.freedesktop.systemd1.Manager", "ReloadOrRestartUnit", ("isc-dhcp-server.service", "replace")).await?;

  Ok(())
}

async fn maybe_update_dhcpd_conf() -> NetworkResult<()> {
  let mut f = File::open(DHCP_MASTER_CONF_FILE)?;
  let mut content = String::new();
  f.read_to_string(&mut content)?;
  
  let include_str = format!("include \"{}\";", DHCP_FILE);
  if !content.contains(include_str.as_str()) {
    warn!("{} does not include the JMS DHCP config. Adding...", DHCP_MASTER_CONF_FILE);
    let mut f = OpenOptions::new().append(true).open(DHCP_MASTER_CONF_FILE)?;
    writeln!(f, "# Automatically added by JMS")?;
    writeln!(f, "{}", include_str)?;
    info!("{} updated!", DHCP_MASTER_CONF_FILE);
  }

  Ok(())
}

#[derive(thiserror::Error, Debug)]
#[error("DHCP Error: {msg}")]
struct DHCPError {
  msg: String
}
