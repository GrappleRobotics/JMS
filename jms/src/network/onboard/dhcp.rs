use std::{fs::File, io::Write, net::{IpAddr, Ipv4Addr}, time::Duration};

use dbus::{Path, arg, nonblock::Proxy};
use handlebars::Handlebars;
use ipnetwork::{IpNetwork, Ipv4Network};
use log::{debug, error, info};
use serde_json::json;
use serde::Serialize;
use tokio::process::Command;

use crate::{arena::station::AllianceStationId, log_expect, network::NetworkResult, utils::{danger::danger_or_err, service_configs::ServiceConfigs, templates}};

const DHCP_FILE: &'static str = "/etc/dhcp/jms-dhcp.conf";

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

pub async fn configure_dhcp(cfg: DHCPConfig, stn_cfgs: &[TeamDHCPConfig]) -> NetworkResult<()> {
  log_expect!(danger_or_err());
  let mut file = File::create(DHCP_FILE)?;

  info!("Generating DHCP file...");
  generate_dhcp_conf(&mut file, &cfg, stn_cfgs).await?;

  info!("Reloading DHCP service...");
  reload_dhcp_service().await?;

  Ok(())
}

async fn generate_dhcp_conf(file: &mut File, cfg: &DHCPConfig, stn_cfgs: &[TeamDHCPConfig]) -> NetworkResult<()> {
  log_expect!(danger_or_err());

  match ServiceConfigs::get("match.dhcp.conf") {
    Some(match_data) => {
      let template_str = std::str::from_utf8(match_data.as_ref())?;
      let mut hbars = Handlebars::new();
      templates::setup(&mut hbars);

      let result = hbars.render_template(
        template_str,
        &json!({
          "admin": cfg,
          "stations": stn_cfgs
        })
      )?;

      file.write_all(result.as_bytes())?;
    },
    None => {
      // TODO: Return Err here
      error!("No ServiceConfig exists: match.dhcp.conf")
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
  // https://www.freedesktop.org/software/systemd/man/org.freedesktop.systemd1.html
  proxy.method_call("org.freedesktop.systemd1.Manager", "StopUnit", ("isc-dhcp-server.service", "replace")).await?;
  // TODO: Maybe update DHCP config to include jms-dhcp.conf if it doesn't already
  
  proxy.method_call("org.freedesktop.systemd1.Manager", "ReloadOrRestartUnit", ("isc-dhcp-server.service", "replace")).await?;

  Ok(())
}