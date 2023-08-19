use std::collections::HashMap;

use jms_networking_lib::NetworkingSettings;
use log::info;
use reqwest::ClientBuilder;
use serde_json::json;

use crate::NetworkConfig;

pub struct UnifiClient {
  client: reqwest::Client,
  uri_base: String
}

impl UnifiClient {
  pub fn new() -> anyhow::Result<Self> {
    let unifi_uri = std::env::var("UNIFI_URI").unwrap_or("https://localhost:8443".to_owned());
    
    let client = ClientBuilder::new()
      .cookie_store(true)
      .user_agent("JMS-Networking")
      .danger_accept_invalid_certs(true)
      .build()?;

    Ok(Self {
      client,
      uri_base: unifi_uri
    })
  }

  pub fn api_url(&self, fragment: &str) -> String {
    format!("{}/api/{}", self.uri_base, fragment)
  }

  pub async fn login(&self, username: &str, password: &str) -> anyhow::Result<()> {
    let result = self.client
      .post(self.api_url("login"))
      .header("Content-Type", "application/json")
      .body(serde_json::to_string(&json!({ "username": username, "password": password }))?)
      .send()
      .await?;
    result.error_for_status()?;
    Ok(())
  }

  pub async fn get(&self, fragment: &str) -> anyhow::Result<serde_json::Value> {
    let result = self.client
      .get(self.api_url(&fragment))
      .send()
      .await?;
    let result = result.error_for_status()?;
    Ok(result.json().await?)
  }

  pub async fn post(&self, fragment: &str, body: serde_json::Value) -> anyhow::Result<()> {
    let result = self.client
      .post(self.api_url(&fragment))
      .json(&body)
      .send()
      .await?;

    result.error_for_status()?;
    Ok(())
  }

  pub async fn get_default(&self, fragment: &str) -> anyhow::Result<serde_json::Value> {
    self.get(&format!("s/default/rest/{}", fragment)).await
  }

  pub async fn put_default(&self, fragment: &str, body: serde_json::Value) -> anyhow::Result<()> {
    let result = self.client
      .put(self.api_url(&format!("s/default/rest/{}", fragment)))
      .json(&body)
      .send()
      .await?;

    result.error_for_status()?;
    Ok(())
  }
}

pub async fn configure_ap_teams(config: &NetworkConfig, settings: &NetworkingSettings) -> anyhow::Result<()> {
  info!("Starting Unifi Update....");
  let client = UnifiClient::new()?;
  client.login(&settings.radio_username, &settings.radio_password).await?;

  let networks = client.get_default("networkconf").await?.get("data").cloned().ok_or(anyhow::anyhow!("No Networkconf Data Given"))?;
  let wlans = client.get_default("wlanconf").await?.get("data").cloned().ok_or(anyhow::anyhow!("No WLAN Conf Data Given"))?;

  // Build VLAN lookup
  let mut network_ids = HashMap::new();
  for network in networks.as_array().ok_or(anyhow::anyhow!("Malformed"))? {
    if let Some(vlan_tag) = network.get("vlan").and_then(|x| x.as_i64()) {
      let id = network.get("_id").and_then(|x| x.as_str()).ok_or(anyhow::anyhow!("Malformed"))?;
      network_ids.insert(id.to_owned(), vlan_tag);
    }
  }

  // Map which vlan belongs to which config
  let mut vlan_configs = HashMap::new();
  vlan_configs.insert(10, config.blue1.clone());
  vlan_configs.insert(20, config.blue2.clone());
  vlan_configs.insert(30, config.blue3.clone());
  vlan_configs.insert(40, config.red1.clone());
  vlan_configs.insert(50, config.red2.clone());
  vlan_configs.insert(60, config.red3.clone());

  // Update the WLANs
  for mut wlan in wlans.as_array().ok_or(anyhow::anyhow!("Malformed!"))?.clone() {
    let network_id = wlan.get("networkconf_id").and_then(|x| x.as_str()).map(|x| x.to_owned());
    let id = wlan.get("_id").and_then(|x| x.as_str()).map(|x| x.to_owned()).ok_or(anyhow::anyhow!("Malformed"))?;
    let wlan_mut = wlan.as_object_mut().ok_or(anyhow::anyhow!("Malformed"))?;
    if let Some(network_id) = network_id {
      if let Some(vlan) = network_ids.get(&network_id) {
        if let Some((team, wpa_key)) = vlan_configs.get(vlan) {
          wlan_mut.insert("name".to_owned(), json!(format!("{}", team)));
          wlan_mut.insert("hide_ssid".to_owned(), json!(true));
          wlan_mut.insert("wlan_band".to_owned(), json!("5g"));
          wlan_mut.insert("wlan_bands".to_owned(), json!(["5g"]));
          wlan_mut.insert("wpa_mode".to_owned(), json!("wpa2"));
          if let Some(wpa_key) = wpa_key {
            wlan_mut.insert("enabled".to_owned(), json!(true));
            wlan_mut.insert("x_passphrase".to_owned(), json!(wpa_key));
          } else {
            wlan_mut.insert("enabled".to_owned(), json!(false));
          }
        }
      }
    }

    client.put_default(&format!("wlanconf/{}", id), wlan).await?;
  }

  info!("Unifi Update Complete!");
  Ok(())
}

pub async fn force_reprovision(settings: &NetworkingSettings) -> anyhow::Result<()> {
  info!("Starting Unifi Reprovision....");
  let client = UnifiClient::new()?;
  client.login(&settings.radio_username, &settings.radio_password).await?;

  let devices = client.get("s/default/stat/device-basic").await?.get("data").cloned().ok_or(anyhow::anyhow!("No Stat Data Given"))?;

  for device in devices.as_array().ok_or(anyhow::anyhow!("Malformed"))? {
    if device.get("type").and_then(|x| x.as_str()).map(|x| x == "uap").ok_or(anyhow::anyhow!("Malformed"))? {
      if let Some(mac) = device.get("mac").and_then(|x| x.as_str()) {
        info!("Reprovisioning {}", mac);
        client.post("s/default/cmd/devmgr", json!({ "cmd": "force-provision", "mac": mac })).await?;
      }
    }
  }
  info!("Unifi Devices Reprovisioned!");
  Ok(())
}