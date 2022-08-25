use sha2::{Sha256, Digest};

use crate::db;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq, Eq)]
pub struct FTAKey(pub String);

impl db::DBSingleton for FTAKey {
  const ID: &'static str = "fta_key";

  fn db_default() -> db::Result<Self> {
    warn!("No FTA Key Set! Creating one now...");
    let key = Self::create(&inquire::Password::new("FTA PIN Number (keep this private)")
      .with_display_mode(inquire::PasswordDisplayMode::Masked)
      .with_validator(inquire::min_length!(4))
      .prompt()?);
    Ok(key)
  }
}

impl FTAKey {
  pub fn create(raw: &str) -> Self {
    let mut hasher = Sha256::new();
    hasher.update(raw);
    Self(base64::encode(hasher.finalize()))
  }

  pub fn validate(&self, other: &str) -> bool {
    return *self == FTAKey::create(other);
  }
}