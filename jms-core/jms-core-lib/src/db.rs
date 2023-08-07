use std::{fmt::Display, str::FromStr};

use jms_base::kv;
use uuid::Uuid;

pub fn generate_id() -> String {
  Uuid::new_v4().to_string()
}

#[async_trait::async_trait]
pub trait Singleton: serde::Serialize + serde::de::DeserializeOwned + Default + Send + Sync {
  const KEY: &'static str;

  fn get(db: &kv::KVConnection) -> anyhow::Result<Self> {
    match db.json_get(&Self::KEY, "$") {
      Ok(v) => Ok(v),
      Err(_) => {
        let default = Self::default();
        default.update(db)?;
        Ok(default)
      }
    }
  }

  fn update(&self, db: &kv::KVConnection) -> anyhow::Result<()> {
    db.json_set(&Self::KEY, "$", &self)
  }

  fn expire(&self, seconds: usize, db: &kv::KVConnection) -> anyhow::Result<()> {
    db.expire(&Self::KEY, seconds)
  } 

  fn delete(db: &kv::KVConnection) -> anyhow::Result<()> {
    db.del(&Self::KEY)
  }
}

#[async_trait::async_trait]
pub trait Table: serde::Serialize + serde::de::DeserializeOwned {
  const PREFIX: &'static str;
  type Err: Display;
  type Id: ToString + FromStr<Err = Self::Err>;
  
  fn id(&self) -> Self::Id;
  fn key(&self) -> String { format!("{}:{}", Self::PREFIX, self.id().to_string()) }

  fn insert(&self, db: &kv::KVConnection) -> anyhow::Result<()> {
    db.json_set(&self.key(), "$", &self)
  }

  fn delete(&self, db: &kv::KVConnection) -> anyhow::Result<()> {
    db.del(&self.key())
  }

  fn delete_by(id: &Self::Id, db: &kv::KVConnection) -> anyhow::Result<()> {
    db.del(&format!("{}:{}", Self::PREFIX, id.to_string()))
  }

  fn get(id: &Self::Id, db: &kv::KVConnection) -> anyhow::Result<Self> {
    db.json_get(&format!("{}:{}", Self::PREFIX, id.to_string()), "$")
  }

  fn ids(db: &kv::KVConnection) -> anyhow::Result<Vec<Self::Id>> {
    let keys = db.keys(&format!("{}:*", Self::PREFIX))?;
    let mut ids = vec![];
    for key in keys {
      ids.push(FromStr::from_str(&key[Self::PREFIX.len() + 1..]).map_err(|e| anyhow::anyhow!("{}", e))?);
    }
    Ok(ids)
  }

  fn all(db: &kv::KVConnection) -> anyhow::Result<Vec<Self>> {
    let mut v = vec![];
    for id in Self::ids(db)? {
      match Self::get(&id, db) {
        Ok(value) => v.push(value),
        _ => ()     // It's since been deleted
      }
    }
    Ok(v)
  }

  fn exists(id: &Self::Id, db: &kv::KVConnection) -> anyhow::Result<bool> {
    db.exists(&format!("{}:{}", Self::PREFIX, id.to_string()))
  }

  fn clear(db: &kv::KVConnection) -> anyhow::Result<()> {
    for id in Self::ids(db)? {
      db.del(&format!("{}:{}", Self::PREFIX, id.to_string())).ok();
    }
    Ok(())
  }

  fn expire(&self, seconds: usize, db: &kv::KVConnection) -> anyhow::Result<()> {
    db.expire(&self.key(), seconds)
  }
}

// Type Bindings

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DBDuration(pub chrono::Duration);

impl serde::Serialize for DBDuration {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_i64(self.0.num_milliseconds())
  }
}

impl schemars::JsonSchema for DBDuration {
  fn schema_name() -> String {
    "DBDuration".to_owned()
  }

  fn is_referenceable() -> bool {
    false
  }

  fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    i64::json_schema(gen).into()
  }
}

impl<'de> serde::Deserialize<'de> for DBDuration {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let ms: i64 = serde::Deserialize::deserialize(deserializer)?;
    Ok(Self(chrono::Duration::milliseconds(ms)))
  }
}

impl From<chrono::Duration> for DBDuration {
  fn from(chrono_dur: chrono::Duration) -> Self {
    Self(chrono_dur)
  }
}
