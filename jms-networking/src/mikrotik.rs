use std::collections::HashMap;

use jms_networking_lib::NetworkingSettings;
use reqwest::ClientBuilder;
use serde_json::json;

use crate::NetworkConfig;

fn api_url(fragment: &str) -> String {
  format!("http://10.0.100.1/rest/{}", fragment)
}

pub async fn configure_firewall(config: &NetworkConfig, settings: &NetworkingSettings) -> anyhow::Result<()> {
  let client = ClientBuilder::new()
    .user_agent("JMS-Networking")
    .danger_accept_invalid_certs(true)
    .build()?;

  let mut cfgs = HashMap::new();
  cfgs.insert("#jms-blue-1", &config.blue1);
  cfgs.insert("#jms-blue-2", &config.blue2);
  cfgs.insert("#jms-blue-3", &config.blue3);
  cfgs.insert("#jms-red-1", &config.red1);
  cfgs.insert("#jms-red-2", &config.red2);
  cfgs.insert("#jms-red-3", &config.red3);

  // Update DHCP Address Pool
  let pool_cfg: serde_json::Value = client.get(&api_url("ip/pool"))
    .basic_auth(&settings.router_username, Some(&settings.router_password))
    .send().await?.error_for_status()?.json().await?;

  for pool in pool_cfg.as_array().ok_or(anyhow::anyhow!("Malformed!"))? {
    for (cfg_comment, cfg_template) in cfgs.iter() {
      if pool.get("comment").and_then(|x| x.as_str()).filter(|x| x.contains(cfg_comment)).is_some() {
        // Matches - patch it
        let id = pool.get(".id").ok_or(anyhow::anyhow!("No ID present for pool."))?.as_str().ok_or(anyhow::anyhow!("Not a string!"))?;
        client.patch(&api_url(&format!("ip/pool/{}", id)))
          .json(&json!({
            "ranges": format!("10.{}.{}.100-10.{}.{}.150", cfg_template.0 / 100, cfg_template.0 % 100, cfg_template.0 / 100, cfg_template.0 % 100)
          }))
          .basic_auth(&settings.router_username, Some(&settings.router_password))
          .send().await?.error_for_status()?;
      }
    }
  }

  // Update DHCP Network Address
  let dhcp_server_network_cfg: serde_json::Value = client.get(&api_url("ip/dhcp-server/network"))
    .basic_auth(&settings.router_username, Some(&settings.router_password))
    .send().await?.error_for_status()?.json().await?;

  for net in dhcp_server_network_cfg.as_array().ok_or(anyhow::anyhow!("Malformed!"))? {
    for (cfg_comment, cfg_template) in cfgs.iter() {
      if net.get("comment").and_then(|x| x.as_str()).filter(|x| x.contains(cfg_comment)).is_some() {
        // Matches - patch it
        let id = net.get(".id").ok_or(anyhow::anyhow!("No ID present for net."))?.as_str().ok_or(anyhow::anyhow!("Not a string!"))?;
        client.patch(&api_url(&format!("ip/dhcp-server/network/{}", id)))
          .json(&json!({
            "address": format!("10.{}.{}.0/24", cfg_template.0 / 100, cfg_template.0 % 100),
            "gateway": format!("10.{}.{}.4", cfg_template.0 / 100, cfg_template.0 % 100)
          }))
          .basic_auth(&settings.router_username, Some(&settings.router_password))
          .send().await?.error_for_status()?;
      }
    }
  }

  // Update IP Address
  let ip_addr_cfg: serde_json::Value = client.get(&api_url("ip/address"))
    .basic_auth(&settings.router_username, Some(&settings.router_password))
    .send().await?.error_for_status()?.json().await?;

  for ip in ip_addr_cfg.as_array().ok_or(anyhow::anyhow!("Malformed!"))? {
    for (cfg_comment, cfg_template) in cfgs.iter() {
      if ip.get("comment").and_then(|x| x.as_str()).filter(|x| x.contains(cfg_comment)).is_some() {
        // Matches - patch it
        let id = ip.get(".id").ok_or(anyhow::anyhow!("No ID present for ip."))?.as_str().ok_or(anyhow::anyhow!("Not a string!"))?;
        client.patch(&api_url(&format!("ip/address/{}", id)))
          .json(&json!({
            "address": format!("10.{}.{}.4/24", cfg_template.0 / 100, cfg_template.0 % 100),
            "network": format!("10.{}.{}.0", cfg_template.0 / 100, cfg_template.0 % 100)
          }))
          .basic_auth(&settings.router_username, Some(&settings.router_password))
          .send().await?.error_for_status()?;
      }
    }
  }

  Ok(())
}