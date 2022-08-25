use std::{fs::File, io::Write, path::Path};

use handlebars::Handlebars;
use ipnetwork::Ipv4Network;
use log::{error, info};
use serde::Serialize;
use serde_json::json;
use sysctl::Sysctl;
use tempfile::NamedTempFile;
use tokio::process::Command;

use crate::{
  arena::station::AllianceStationId,
  log_expect,
  network::NetworkResult,
  utils::{danger::danger_or_err, service_configs::ServiceConfigs, templates},
};

#[derive(Serialize)]
pub struct FirewallConfig {
  pub iface: String,
  pub router: Option<Ipv4Network>,
  pub server: Option<Ipv4Network>,
}

#[derive(Serialize)]
pub struct WanConfig {
  pub iface: String,
  pub access: bool,
}

#[derive(Serialize)]
pub struct TeamFirewallConfig {
  pub station: AllianceStationId,
  pub team: Option<u16>,
  pub cfg: FirewallConfig,
}

pub async fn configure_firewall(
  wan_cfg: WanConfig,
  admin_cfg: FirewallConfig,
  stn_cfgs: &[TeamFirewallConfig],
) -> NetworkResult<()> {
  log_expect!(danger_or_err());
  let mut file = NamedTempFile::new()?;

  info!("Generating firewall rule file...");
  generate_firewall_rules(file.as_file_mut(), wan_cfg, admin_cfg, stn_cfgs).await?;

  info!("Reloading firewall...");
  reload_firewall(file.path()).await?;

  info!("Firewall ready!");
  Ok(())
}

async fn generate_firewall_rules(
  file: &mut File,
  wan_cfg: WanConfig,
  admin_cfg: FirewallConfig,
  stn_cfgs: &[TeamFirewallConfig],
) -> NetworkResult<()> {
  log_expect!(danger_or_err());

  match ServiceConfigs::get("match.firewall.rules") {
    Some(match_data) => {
      let template_str = std::str::from_utf8(match_data.as_ref())?;
      let mut hbars = Handlebars::new();
      templates::setup(&mut hbars);

      let result = hbars.render_template(
        template_str,
        &json!({
          "wan": wan_cfg,
          "admin": admin_cfg,
          "stations": stn_cfgs,
        }),
      )?;

      file.write_all(result.as_bytes())?;
    }
    None => {
      error!("No ServiceConfig exists: match.firewall.conf");
      // TODO: Return Err
    }
  }
  Ok(())
}

async fn reload_firewall(file_path: &Path) -> NetworkResult<()> {
  log_expect!(danger_or_err());

  // Enable ipv4 forwarding
  let ctl = sysctl::Ctl::new("net.ipv4.ip_forward")?;
  ctl.set_value_string("1")?;

  // Write iptables rules
  let result = Command::new("iptables-restore").arg(file_path).output().await?;
  if !result.status.success() {
    let err = std::str::from_utf8(&result.stderr)?;
    let out = std::str::from_utf8(&result.stdout)?;

    error!("Could not apply firewall rules: stdout: {}", out);
    error!("Could not apply firewall rules: stderr: {}", err);
  }

  Ok(())
}
