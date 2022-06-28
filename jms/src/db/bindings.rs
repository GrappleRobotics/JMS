use std::convert::TryFrom;

use chrono::{DateTime, Duration, NaiveDateTime};

use super::TableType;
use super::types::Key;

// A shallow DB type used to bind to another table as a foreign key,
// to be reloaded upon deserialisation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "ShallowInner<T>", into = "ShallowInner<T>")]
#[serde(bound = "T: TableType + Clone")]
pub struct Shallow<T>(pub T);

impl<T> From<T> for Shallow<T> {
  fn from(t: T) -> Self {
    Self(t)
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ShallowInner<T> {
  FK(Vec<u8>),
  WRAPPED(T)
}

impl<T: TableType> From<Shallow<T>> for ShallowInner<T> {
  fn from(shallow: Shallow<T>) -> Self {
    match shallow.0.id() {
      Some(id) => ShallowInner::FK(id.as_ref().to_vec()),
      None => ShallowInner::WRAPPED(shallow.0),
    }
  }
}

impl<T: TableType> TryFrom<ShallowInner<T>> for Shallow<T> {
  type Error = anyhow::Error;

  fn try_from(value: ShallowInner<T>) -> Result<Self, Self::Error> {
    match value {
      ShallowInner::FK(id) => {
        let id: &[u8] = id.as_ref();
        let coerced_id = T::Id::from_raw(id);
        let got = T::table(super::database())?.get(coerced_id)?;
        match got {
          Some(t) => Ok(Shallow(t)),
          None => Err(anyhow::anyhow!("No ID {:?} exists in table {}", id, T::TABLE)),
        }
      },
      ShallowInner::WRAPPED(t) => Ok(Shallow(t)),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DBDateTime(pub NaiveDateTime);

impl serde::Serialize for DBDateTime {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    serializer.serialize_i64(self.0.timestamp())
  }
}

impl schemars::JsonSchema for DBDateTime {
  fn schema_name() -> String {
    "DBDatetime".to_owned()
  }

  fn is_referenceable() -> bool {
    false
  }

  fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    i64::json_schema(gen).into()
  }
}

impl<'de> serde::Deserialize<'de> for DBDateTime {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    let ms: i64 = serde::Deserialize::deserialize(deserializer)?;
    Ok(Self(NaiveDateTime::from_timestamp(ms, 0)))
  }
}

impl<T: chrono::TimeZone> From<DateTime<T>> for DBDateTime {
  fn from(t: DateTime<T>) -> Self {
    Self(t.naive_utc())
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DBDuration(pub Duration);

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
    Ok(Self(Duration::milliseconds(ms)))
  }
}