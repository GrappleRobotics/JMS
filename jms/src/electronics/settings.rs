use crate::config::Interactive;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DMXLightSettings {
  pub base_address: usize,
  pub n_modules: usize,
}

impl DMXLightSettings {
  pub fn new(base: usize, n: usize) -> Self {
    Self { base_address: base, n_modules: n }
  }

  pub async fn interactive(name: &str, default_address: usize) -> anyhow::Result<Self> {
    println!("Light - {}", name);
    let base_address = inquire::CustomType::<usize>::new("Base DMX Address").with_default( (default_address, &|x| x.to_string()) ).prompt()?;
    let n_modules = inquire::CustomType::<usize>::new("Number of Modules").with_default( (8, &|x| x.to_string()) ).prompt()?;

    Ok(Self { base_address, n_modules })
  }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct LightingConfig {
  pub blue: [DMXLightSettings; 3],
  pub red: [DMXLightSettings; 3],
  pub scoring_table: [DMXLightSettings; 2]
}

#[async_trait::async_trait]
impl Interactive for LightingConfig {
  async fn interactive() -> anyhow::Result<Self> {
    let blue = [
      DMXLightSettings::interactive("Blue 1", 150).await?,
      DMXLightSettings::interactive("Blue 2", 200).await?,
      DMXLightSettings::interactive("Blue 3", 250).await?,
    ];

    let red = [
      DMXLightSettings::interactive("Red 1", 350).await?,
      DMXLightSettings::interactive("Red 2", 400).await?,
      DMXLightSettings::interactive("Red 3", 450).await?,
    ];

    let scoring_table = [
      DMXLightSettings::interactive("Scoring Table (Blue)", 100).await?,
      DMXLightSettings::interactive("Scoring Table (Red)", 300).await?,
    ];

    Ok(Self { blue, red, scoring_table })
  }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InnerElectronicsSettings {
  pub lighting: LightingConfig
}

#[async_trait::async_trait]
impl Interactive for InnerElectronicsSettings {
  async fn interactive() -> anyhow::Result<Self> {
    Ok(Self {
      lighting: LightingConfig::interactive().await?
    })
  }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ElectronicsSettings(pub Option<InnerElectronicsSettings>);

#[async_trait::async_trait]
impl Interactive for ElectronicsSettings {
  async fn interactive() -> anyhow::Result<Self> {
    let use_electronics = inquire::Confirm::new("Configure Field Electronics (incl. lighting)?")
      .with_default(false)
      .prompt()?;
    
    match use_electronics {
      true => Ok(Self(Some(InnerElectronicsSettings::interactive().await?))),
      false => Ok(Self(None))
    }
  }
}