mod team;
use std::fmt::Debug;

use chrono::{Duration, NaiveDateTime};
use diesel::{sql_types::{BigInt, Integer, Text}, types::{FromSql, ToSql}};
pub use team::*;

mod matches;
pub use matches::*;

mod event_details;
pub use event_details::*;

mod schedule;
pub use schedule::*;

// SQL-mapped vector (for sqlite)

#[derive(AsExpression, Debug, serde::Deserialize, serde::Serialize, FromSqlRow, Clone)]
#[sql_type = "Text"]
pub struct SQLJsonVector<T>(pub Vec<T>);

impl<'a, T, DB> FromSql<Text, DB> for SQLJsonVector<T>
where 
  DB: diesel::backend::Backend,
  T: serde::de::DeserializeOwned,
  String: FromSql<Text, DB>
{
  fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
    let t = String::from_sql(bytes)?;
    let t: Vec<T> = serde_json::from_str(&t)?;
    Ok(Self(t))
  }
}

impl<T, DB> ToSql<Text, DB> for SQLJsonVector<T>
where
  DB: diesel::backend::Backend,
  T: serde::Serialize + std::fmt::Debug,
  String: ToSql<Text, DB>
{
  fn to_sql<W: std::io::Write>(&self, out: &mut diesel::serialize::Output<W, DB>) -> diesel::serialize::Result {
    let s = serde_json::to_string(&self.0)?;
    String::to_sql(&s, out)
  }
}

// SQL Enums as text (for sqlite)

#[macro_export]
macro_rules! as_item {
    ($i:item) => { $i };
}

// Map an enum to TEXT in SQL
#[macro_export]
macro_rules! sql_mapped_enum {
  ($name:ident, $($variants:tt)+) => {
    crate::as_item! {
      #[derive(Debug, strum_macros::EnumString, strum_macros::ToString, AsExpression, FromSqlRow, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
      #[sql_type = "diesel::sql_types::Text"]
      pub enum $name {
        $($variants)*
      }
    }

    impl<DB> diesel::types::ToSql<diesel::sql_types::Text, DB> for $name
    where
      DB: diesel::backend::Backend,
      String: diesel::types::ToSql<diesel::sql_types::Text, DB>
    {
      fn to_sql<W: std::io::Write>(&self, out: &mut diesel::serialize::Output<W, DB>) -> diesel::serialize::Result {
        String::to_sql(&self.to_string(), out)
      }
    }

    impl<DB> diesel::types::FromSql<diesel::sql_types::Text, DB> for $name
    where
      DB: diesel::backend::Backend,
      String: diesel::types::FromSql<diesel::sql_types::Text, DB>
    {
      fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        use std::str::FromStr;
        let t = String::from_sql(bytes)?;
        Ok(Self::from_str(&t)?)
      }
    }
  }
}

// SQL-mapped chrono types (for sqlite + serde)

fn serialize_naivedatetime<S>(ndt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer
{
  serializer.serialize_str(&ndt.format("%F %T.%f").to_string())
}

fn deserialize_naivedatetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
  D: serde::Deserializer<'de>
{
  let s: String = serde::Deserialize::deserialize(deserializer)?;
  NaiveDateTime::parse_from_str(&s, "%F %T.%f").map_err(serde::de::Error::custom)
}

#[derive(AsExpression, Debug, FromSqlRow, Clone)]
#[sql_type = "BigInt"]
pub struct SQLDuration(pub Duration);

impl serde::Serialize for SQLDuration {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer
  {
    serializer.serialize_i64(self.0.num_milliseconds())
  }
}

impl<'de> serde::Deserialize<'de> for SQLDuration {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>
  {
    let ms: i64 = serde::Deserialize::deserialize(deserializer)?;
    Ok(Self(Duration::milliseconds(ms)))
  }
}

impl<'a, DB> FromSql<BigInt, DB> for SQLDuration
where 
  DB: diesel::backend::Backend,
  i64: FromSql<BigInt, DB>
{
  fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
    let t = Duration::milliseconds(i64::from_sql(bytes)?);
    Ok(Self(t))
  }
}

impl<DB> ToSql<BigInt, DB> for SQLDuration
where
  DB: diesel::backend::Backend,
  i64: ToSql<BigInt, DB>
{
  fn to_sql<W: std::io::Write>(&self, out: &mut diesel::serialize::Output<W, DB>) -> diesel::serialize::Result {
    i64::to_sql(&self.0.num_milliseconds(), out)
  }
}