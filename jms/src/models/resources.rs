use crate::{arena::resource::ResourceRequirements, db::{self, TableType}};

// Resource Requirements get stored in the database to persist. Configured from the event wizard

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DBResourceRequirements(pub Option<ResourceRequirements>);

impl db::TableType for DBResourceRequirements {
  const TABLE: &'static str = "resource_requirements";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    Some(1.into())
  }

  fn set_id(&mut self, _id: Self::Id) {}
}

impl DBResourceRequirements {
  pub fn get(store: &db::Store) -> db::Result<Self> {
    let first = Self::table(store)?.first()?;

    match first {
      Some(rr) => Ok(rr),
      None => {
        let mut rr = DBResourceRequirements(None);
        rr.insert(store)?;
        Ok(rr)
      }
    }
  }
}