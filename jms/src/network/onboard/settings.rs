use crate::{
  config::Interactive,
  network::{onboard::netlink, radio::settings::FieldRadioSettings},
};

use super::netlink::LinkMetadata;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct OnboardNetworkSettings {
  pub iface_wan: String,
  pub iface_admin: String,
  pub ifaces_blue: Vec<String>,
  pub ifaces_red: Vec<String>,
  pub radio: Option<FieldRadioSettings>,
}

pub fn select_iface<'a>(message: &str, vlan: u16, ifaces: &'a Vec<LinkMetadata>) -> anyhow::Result<&'a LinkMetadata> {
  let options: Vec<String> = ifaces.iter().map(|i| format!("{}", i)).collect();
  let options_r: Vec<&str> = options.iter().map(|s| s.as_str()).collect();

  let mut inq = inquire::Select::new(message, &options_r[..]);
  if vlan != 0 {
    let default = ifaces.iter().enumerate().find(|(_, i)| i.vlan == Some(vlan));
    if let Some((idx, _)) = default {
      inq.starting_cursor = idx;
    }
  }
  let selection = inq.prompt()?.index;
  Ok(&ifaces[selection])
}

#[async_trait::async_trait]
impl Interactive for OnboardNetworkSettings {
  async fn interactive() -> anyhow::Result<Self> {
    let handle = netlink::handle()?;
    let mut ifaces = netlink::get_all_ifaces(&handle).await?;
    ifaces.sort_by(|a, b| a.name.cmp(&b.name));

    let iface_wan = select_iface("WAN Interface (VLAN 150)", 150, &ifaces)?.name.clone();
    let iface_admin = select_iface("Admin Interface (VLAN 100)", 100, &ifaces)?.name.clone();

    let ifaces_blue: Vec<String> = (1..=3)
      .map(|x| {
        Ok(
          select_iface(
            &format!("Blue {} Interface (VLAN {})", x, x * 10),
            x * 10 as u16,
            &ifaces,
          )?
          .name
          .clone(),
        )
      })
      .collect::<anyhow::Result<Vec<String>>>()?;

    let ifaces_red: Vec<String> = (1..=3)
      .map(|x| {
        Ok(
          select_iface(
            &format!("Red {} Interface (VLAN {})", x, 30 + x * 10),
            30 + x * 10 as u16,
            &ifaces,
          )?
          .name
          .clone(),
        )
      })
      .collect::<anyhow::Result<Vec<String>>>()?;

    let use_radio = inquire::Confirm::new("Do you want to configure a Field Wireless Radio?")
      .with_default(true)
      .prompt()?;
    let radio = match use_radio {
      true => Some(FieldRadioSettings::interactive().await?),
      false => None,
    };

    Ok(Self {
      iface_wan,
      iface_admin,
      ifaces_blue,
      ifaces_red,
      radio,
    })
  }
}
