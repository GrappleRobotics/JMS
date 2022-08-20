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
}

impl DBResourceRequirements {
  pub fn get(store: &db::Store) -> db::Result<Self> {
    match Self::first(store)? {
      Some(rr) => Ok(rr),
      None => {
        let mut rr = DBResourceRequirements(None);
        rr.insert(store)?;
        Ok(rr)
      }
    }
  }
}