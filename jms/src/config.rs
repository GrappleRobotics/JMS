use std::{
  fs::{self, File},
  io::ErrorKind,
  path::Path,
  str::FromStr,
};

use crate::network::NetworkSettings;

use log::warn;
use strum::IntoEnumIterator;

const CONFIG_PATH: &'static str = "/etc/jms/jms.yml";

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct JMSSettings {
  pub network: NetworkSettings,
}

#[async_trait::async_trait]
impl Interactive for JMSSettings {
  async fn interactive() -> anyhow::Result<Self> {
    Ok(Self {
      network: NetworkSettings::interactive().await?,
    })
  }
}

impl JMSSettings {
  pub fn load_config() -> anyhow::Result<Self> {
    let file = File::open(CONFIG_PATH)?;
    Ok(serde_yaml::from_reader(&file)?)
  }

  pub async fn create_config() -> anyhow::Result<Self> {
    let parent = Path::new(CONFIG_PATH).parent();
    if let Some(parent) = parent {
      fs::create_dir_all(parent)?;
    }

    let settings = JMSSettings::interactive().await?;
    let f = File::create(CONFIG_PATH)?;
    serde_yaml::to_writer(f, &settings)?;

    info!("Configuration written!");
    Ok(settings)
  }

  pub async fn load_or_create_config(force_new: bool) -> anyhow::Result<Self> {
    if force_new {
      return Self::create_config().await;
    }

    match Self::load_config() {
      Ok(s) => {
        info!("Loaded JMS Config");
        Ok(s)
      }
      Err(e) => match e.downcast_ref::<std::io::Error>() {
        Some(ioe) => match ioe.kind() {
          ErrorKind::NotFound => {
            warn!("{} does not exist", CONFIG_PATH);
            warn!("Creating new configuration interactively");

            Self::create_config().await
          }
          _ => Err(e),
        },
        None => Err(e),
      },
    }
  }
}

#[async_trait::async_trait]
pub trait Interactive
where
  Self: Sized,
{
  async fn interactive() -> anyhow::Result<Self>;
}

pub fn enum_interactive<E, I, T>(message: &str) -> anyhow::Result<T>
where
  E: IntoEnumIterator<Iterator = I>,
  I: Iterator<Item = T>,
  T: ToString + FromStr,
  <T as FromStr>::Err: std::fmt::Debug,
{
  let options: Vec<String> = E::iter().map(|t| t.to_string()).collect();
  let options_ref: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
  let result = inquire::Select::new(message, &options_ref[..]).prompt()?.value;
  Ok(T::from_str(result.as_str()).unwrap())
}
