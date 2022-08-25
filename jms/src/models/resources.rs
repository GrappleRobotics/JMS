use crate::{arena::resource::ResourceRequirements, db};

// Resource Requirements get stored in the database to persist. Configured from the event wizard

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DBResourceRequirements(pub Option<ResourceRequirements>);

impl db::DBSingleton for DBResourceRequirements {
  const ID: &'static str = "resource_requirements";

  fn db_default() -> db::Result<Self> {
    Ok(Self(None))
  }
}
