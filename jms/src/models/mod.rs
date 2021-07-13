mod team;
use std::fmt::Debug;

use diesel::{sql_types::Text, types::{FromSql, ToSql}};
pub use team::*;

mod matches;
pub use matches::*;

#[derive(AsExpression, Debug, serde::Deserialize, serde::Serialize, FromSqlRow, Clone)]
#[sql_type = "Text"]
pub struct SQLJsonVector<T>(pub Vec<T>);

impl<'a, T, DB> FromSql<Text, DB> for SQLJsonVector<T>
where 
  DB: diesel::backend::Backend,
  T: serde::de::DeserializeOwned,
  String: FromSql<T, DB>
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