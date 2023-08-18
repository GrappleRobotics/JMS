use jms_core_lib::db::Singleton;

#[derive(jms_macros::Updateable)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct BackupSettings {
  pub s3_endpoint: String,
  pub s3_access_key: Option<String>,
  pub s3_secret_access_key: Option<String>,
  pub s3_bucket: Option<String>,

  pub file_target_dir: Option<String>,
}

impl Default for BackupSettings {
  fn default() -> Self {
    Self {
      s3_endpoint: "s3.ap-southeast-2.amazonaws.com".to_owned(),
      s3_access_key: None,
      s3_secret_access_key: None,
      s3_bucket: None,
      file_target_dir: None
    }
  }
}

impl Singleton for BackupSettings {
  const KEY: &'static str = "db:backup:settings";
}

#[jms_macros::service]
pub trait JMSBackupRPC {
  async fn backup_now() -> Result<(), String>;
  async fn backup_to() -> Result<Vec<u8>, String>;
  async fn restore(backup: Vec<u8>) -> Result<(), String>;
}