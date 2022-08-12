use sha2::{Sha256, Digest};

use crate::db::{self, TableType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq, Eq)]
pub struct FTAKey(pub String);

impl db::TableType for FTAKey {
  const TABLE: &'static str = "fta_key";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    Some(1.into())
  }
}

impl FTAKey {
  pub fn create(raw: &str) -> Self {
    let mut hasher = Sha256::new();
    hasher.update(raw);
    Self(base64::encode(hasher.finalize()))
  }

  pub fn get(store: &db::Store) -> db::Result<Self> {
    let first = Self::table(store)?.first()?;
    match first {
      Some(key) => Ok(key),
      None => {
        warn!("No FTA Key Set! Creating one now...");
        let mut key = Self::create(&inquire::Password::new("FTA PIN Number (keep this private)")
          .with_display_mode(inquire::PasswordDisplayMode::Masked)
          .with_validator(inquire::min_length!(4))
          .prompt()?);
        key.insert(store)?;
        Ok(key)
      }
    }
  }

  pub fn validate(&self, other: &str) -> bool {
    return *self == FTAKey::create(other);
  }
}