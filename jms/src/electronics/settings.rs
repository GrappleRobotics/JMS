use crate::config::Interactive;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InnerElectronicsSettings {
  pub port: String,
  pub baud: usize
}

#[async_trait::async_trait]
impl Interactive for InnerElectronicsSettings {
  async fn interactive() -> anyhow::Result<Self> {
    let all_ports = tokio_serial::available_ports()?;
    let port_names: Vec<&str> = all_ports.iter().map(|i| i.port_name.as_str()).collect();
    let port = inquire::Select::new("Select Serial Port for Master Node", &port_names[..]).prompt()?.value;
    let baud = inquire::CustomType::<usize>::new("Baud Rate?").with_default( (115200, &|x| x.to_string())).prompt()?;

    Ok(InnerElectronicsSettings {
      port, baud
    })
  }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ElectronicsSettings(pub Option<InnerElectronicsSettings>);

#[async_trait::async_trait]
impl Interactive for ElectronicsSettings {
  async fn interactive() -> anyhow::Result<Self> {
    let use_electronics = inquire::Confirm::new("Configure Field Electronics?")
      .with_default(false)
      .prompt()?;
    
    match use_electronics {
      true => Ok(Self(Some(InnerElectronicsSettings::interactive().await?))),
      false => Ok(Self(None))
    }
  }
}