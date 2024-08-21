use jms_networking_lib::NetworkingSettings;
use log::info;
use reqwest::ClientBuilder;
use serde_json::json;
use tokio::time::Duration;

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

  pub fn apiv2_url(&self, fragment: &str) -> String {
    format!("{}/v2/api/{}", self.uri_base, fragment)
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

  pub async fn get_default_v2(&self, fragment: &str) -> anyhow::Result<serde_json::Value> {
    let result = self.client
      .get(self.apiv2_url(&format!("site/default/{}", fragment)))
      .send()
      .await?;
    let result = result.error_for_status()?;
    Ok(result.json().await?)
  }

  pub async fn post_default_v2(&self, fragment: &str, body: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let result = self.client
      .post(self.apiv2_url(&format!("site/default/{}", fragment)))
      .json(&body)
      .send()
      .await?;
    let result = result.error_for_status()?;
    Ok(result.json().await?)
  }

  pub async fn put_default_v2(&self, fragment: &str, body: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let result = self.client
      .put(self.apiv2_url(&format!("site/default/{}", fragment)))
      .json(&body)
      .send()
      .await?;
    let result = result.error_for_status()?;
    Ok(result.json().await?)
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

  pub async fn post_default(&self, fragment: &str, body: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let result = self.client
      .post(self.api_url(&format!("s/default/rest/{}", fragment)))
      .json(&body)
      .send()
      .await?;

    let result = result.error_for_status()?;
    Ok(result.json().await?)
  }
}

pub struct UnifiNetwork {
  vlan: usize,
  ssid: String,
  passkey: Option<String>,
  hidden: bool,
  band: String,
  ap_group: String,
  enabled: bool,
}

pub async fn provision_controller(client: &UnifiClient) -> anyhow::Result<()> {
  info!("Provisioning Unifi Controller for the first time....");

  client.post("cmd/sitemgr", json!(
    {"cmd":"add-default-admin","name":"FTA","email":"fta@jms.local","x_password":"jmsR0cks"}
  )).await?;

  client.post("set/setting/super_identity", json!({ "name": "JMS" })).await?;
  client.post("set/setting/country", json!({ "code": "36" })).await?;   // Australia, TODO: Update to reflect actual country
  client.post("set/setting/locale", json!({ "timezone": iana_time_zone::get_timezone().unwrap_or("Australia/Sydney".to_owned()) })).await?;
  client.post("set/setting/super_mgmt", json!({ "autobackup_enabled":false, "backup_to_cloud_enabled":false })).await?;
  client.post("set/setting/mgmt", json!({"x_ssh_username":"FTA","x_ssh_password":"jmsR0cks"})).await?;
  client.post("cmd/system", json!({"cmd":"set-installed"})).await?;

  tokio::time::sleep(Duration::from_secs(5)).await;

  // Disable Meshing
  let settings_json = client.get_default("get/setting").await?;
  let all_settings = settings_json.get("data").and_then(|data| data.as_array()).ok_or(anyhow::anyhow!("Malformed!"))?;
  for setting in all_settings {
    if setting.get("key").map(|key| key.as_str() == Some("connectivity")) == Some(true) {
      let id = setting.get("id").and_then(|id| id.as_str()).ok_or(anyhow::anyhow!("Malformed!"))?;
      client.put_default("group/setting", json!({
        "objects":[{
          "id":id,
          "data":{
            "_id":id,
            "key":"connectivity",
            "enabled":false
          }
        }]
      })).await?;
    }
  }
  
  info!("Unifi Controller provisioned!");

  Ok(())
}

pub async fn configure(config: &NetworkConfig, settings: &NetworkingSettings) -> anyhow::Result<()> {
  info!("Starting Unifi Update....");
  let client = UnifiClient::new()?;
  if let Err(_) = client.login(&settings.radio_username, &settings.radio_password).await {
    provision_controller(&client).await.ok();
  }

  let mut network_confs = vec![ ];

  // TODO: Disable wireless meshing.

  if let Some(admin_ssid) = settings.admin_ssid.clone() {
    network_confs.push(UnifiNetwork {
      ap_group: "Admin APs".to_owned(),
      band: "2g".to_owned(),
      ssid: admin_ssid,
      passkey: settings.admin_password.clone(),
      hidden: true,
      vlan: 1,
      enabled: true
    });
  }

  if let Some(guest_ssid) = settings.guest_ssid.clone() {
    network_confs.push(UnifiNetwork {
      ap_group: "Team Network APs".to_owned(),
      band: "2g".to_owned(),
      ssid: guest_ssid,
      passkey: settings.guest_password.clone(),
      hidden: false,
      vlan: 200,
      enabled: true
    });
  }

  for (vlan, (team, passkey)) in &[(10, &config.blue1), (20, &config.blue2), (30, &config.blue3), (40, &config.red1), (50, &config.red2), (60, &config.red3)] {
    network_confs.push(UnifiNetwork {
      vlan: *vlan,
      ssid: team.map(|team| format!("{}", team)).unwrap_or(format!("unoccupied-{}", vlan)),
      passkey: passkey.to_owned(),
      hidden: true,
      band: "5g".to_owned(),
      ap_group: "Field APs".to_owned(),
      enabled: passkey.is_some()
    });
  }

  let networks = client.get_default("networkconf").await?.get("data")
    .cloned().ok_or(anyhow::anyhow!("No Networkconf Data Given"))?
    .as_array().ok_or(anyhow::anyhow!("Malformed!"))?.clone();

  let wlans = client.get_default("wlanconf").await?.get("data")
    .cloned().ok_or(anyhow::anyhow!("No WLAN Conf Data Given"))?
    .as_array().ok_or(anyhow::anyhow!("Malformed"))?.clone();

  for netconf in network_confs {
    let apgroups = client.get_default_v2("apgroups").await?
      .clone().as_array().ok_or(anyhow::anyhow!("Malformed"))?.clone();

    // Network 1 is a special case - we have to use the default since we use VLAN1 for ADMIN, as opposed to VLAN100 (due to a unifi restriction)
    let candidate_network = match netconf.vlan {
      1 => networks.iter().find(|net| net.get("name").and_then(|x| x.as_str()) == Some("Default")),
      _ => networks.iter().find(|net| net.get("vlan").and_then(|x| x.as_i64()) == Some(netconf.vlan as i64))
    }.and_then(|net| net.get("_id")).and_then(|id| id.as_str());
    let candidate_network_id = match candidate_network {
      Some(id) => id.to_owned(),
      None => {
        let resp = client.post_default("networkconf", json!({
          "is_nat": true,
          "vlan": netconf.vlan,
          "purpose": "vlan-only",
          "igmp_snooping": false,
          "name": format!("VLAN{}", netconf.vlan),
          "dhcpguard_enabled": false,
          "vlan_enabled": true,
          "enabled": true,
        })).await?;
        let id = resp.get("data").ok_or(anyhow::anyhow!("No Network Data Given"))?
          .as_array().ok_or(anyhow::anyhow!("Malformed!"))?
          .get(0).ok_or(anyhow::anyhow!("Malformed!"))?
          .get("_id").ok_or(anyhow::anyhow!("No Network ID Given!"))?;
        id.as_str().ok_or(anyhow::anyhow!("Malformed!"))?.to_owned()
      }
    };

    let candidate_apgroup = apgroups.iter().find(|apgroup| apgroup.get("name").and_then(|x| x.as_str()) == Some(&netconf.ap_group)).and_then(|apgroup| apgroup.get("_id")).and_then(|id| id.as_str());
    let candidate_apgroup_id = match candidate_apgroup {
      Some(id) => id.to_owned(),
      None => {
        let resp = client.post_default_v2("apgroups", json!({
          "device_macs": [],
          "name": netconf.ap_group
        })).await?;
        let id = resp.get("_id").ok_or(anyhow::anyhow!("No APGroup ID Given!"))?;
        id.as_str().ok_or(anyhow::anyhow!("Malformed!"))?.to_owned()
      }
    };

    let candidate_wlan = wlans.iter().find(|wlan| wlan.get("networkconf_id").and_then(|x| x.as_str()) == Some(&candidate_network_id));
    match candidate_wlan.cloned() {
      Some(mut wlan) => {
        let id = wlan.get("_id").and_then(|x| x.as_str()).map(|x| x.to_owned()).ok_or(anyhow::anyhow!("Malformed"))?;
        let wlan_mut = wlan.as_object_mut().ok_or(anyhow::anyhow!("Malformed"))?;
        wlan_mut.insert("name".to_owned(), json!(netconf.ssid));
        wlan_mut.insert("hide_ssid".to_owned(), json!(netconf.hidden));
        wlan_mut.insert("wlan_band".to_owned(), json!(netconf.band));
        wlan_mut.insert("wlan_bands".to_owned(), json!([netconf.band]));
        wlan_mut.insert("wpa_mode".to_owned(), json!("wpa2"));
        wlan_mut.insert("enabled".to_owned(), json!(netconf.enabled));
        wlan_mut.insert("ap_group_ids".to_owned(), json!([candidate_apgroup_id]));
        if let Some(wpa_key) = &netconf.passkey {
          wlan_mut.insert("security".to_owned(), json!("wpapsk"));
          wlan_mut.insert("x_passphrase".to_owned(), json!(wpa_key));
        } else {
          wlan_mut.insert("security".to_owned(), json!("open"));
        }
        client.put_default(&format!("wlanconf/{}", id), wlan).await?;
      },
      None => {
        let mut json_body = json!({
          "networkconf_id": candidate_network_id,
          "name": netconf.ssid,
          "hide_ssid": netconf.hidden,
          "wlan_band": netconf.band,
          "wlan_bands": [netconf.band],
          "wpa_mode": "wpa2",
          "enabled": netconf.enabled,
          "security": if netconf.passkey.is_some() { "wpapsk" } else { "open" },
          "ap_group_ids": [candidate_apgroup_id],
          "minrate_setting_preference": "auto"
        });

        if netconf.passkey.is_some() {
          json_body.as_object_mut().unwrap().insert("x_passphrase".to_owned(), json!(netconf.passkey.unwrap_or("".to_owned())));
        }

        client.post_default("wlanconf", json_body).await?;
      },
    }
  }

  info!("Unifi Update Complete!");

  Ok(())
}

// pub async fn configure_ap_teams(config: &NetworkConfig, settings: &NetworkingSettings) -> anyhow::Result<()> {
//   info!("Starting Unifi Update....");
//   let client = UnifiClient::new()?;
//   client.login(&settings.radio_username, &settings.radio_password).await?;

//   let networks = client.get_default("networkconf").await?.get("data").cloned().ok_or(anyhow::anyhow!("No Networkconf Data Given"))?;
//   let wlans = client.get_default("wlanconf").await?.get("data").cloned().ok_or(anyhow::anyhow!("No WLAN Conf Data Given"))?;

//   // Build VLAN lookup
//   let mut network_ids = HashMap::new();
//   for network in networks.as_array().ok_or(anyhow::anyhow!("Malformed"))? {
//     if let Some(vlan_tag) = network.get("vlan").and_then(|x| x.as_i64()) {
//       let id = network.get("_id").and_then(|x| x.as_str()).ok_or(anyhow::anyhow!("Malformed"))?;
//       network_ids.insert(id.to_owned(), vlan_tag);
//     }
//   }

//   // Map which vlan belongs to which config
//   let mut vlan_configs = HashMap::new();
//   vlan_configs.insert(10, config.blue1.clone());
//   vlan_configs.insert(20, config.blue2.clone());
//   vlan_configs.insert(30, config.blue3.clone());
//   vlan_configs.insert(40, config.red1.clone());
//   vlan_configs.insert(50, config.red2.clone());
//   vlan_configs.insert(60, config.red3.clone());

//   // Update the WLANs
//   for mut wlan in wlans.as_array().ok_or(anyhow::anyhow!("Malformed!"))?.clone() {
//     let network_id = wlan.get("networkconf_id").and_then(|x| x.as_str()).map(|x| x.to_owned());
//     let id = wlan.get("_id").and_then(|x| x.as_str()).map(|x| x.to_owned()).ok_or(anyhow::anyhow!("Malformed"))?;
//     let wlan_mut = wlan.as_object_mut().ok_or(anyhow::anyhow!("Malformed"))?;
//     if let Some(network_id) = network_id {
//       if let Some(vlan) = network_ids.get(&network_id) {
//         if let Some((team, wpa_key)) = vlan_configs.get(vlan) {
//           wlan_mut.insert("name".to_owned(), json!(team.map(|x| format!("{}", x)).unwrap_or(format!("unoccupied-{}", vlan))));
//           wlan_mut.insert("hide_ssid".to_owned(), json!(true));
//           wlan_mut.insert("wlan_band".to_owned(), json!("5g"));
//           wlan_mut.insert("wlan_bands".to_owned(), json!(["5g"]));
//           wlan_mut.insert("wpa_mode".to_owned(), json!("wpa2"));
//           if let Some(wpa_key) = wpa_key {
//             wlan_mut.insert("enabled".to_owned(), json!(true));
//             wlan_mut.insert("x_passphrase".to_owned(), json!(wpa_key));
//           } else {
//             wlan_mut.insert("enabled".to_owned(), json!(false));
//           }
//         }
//       }
//     }

//     client.put_default(&format!("wlanconf/{}", id), wlan).await?;
//   }

//   info!("Unifi Update Complete!");
//   Ok(())
// }

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