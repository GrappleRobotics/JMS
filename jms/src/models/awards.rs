use crate::db;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AwardRecipient {
  pub team: Option<usize>,
  pub awardee: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Award {
  pub id: Option<u64>,
  pub name: String,
  pub recipients: Vec<AwardRecipient>,
}

impl db::TableType for Award {
  const TABLE: &'static str = "awards";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    self.id.map(|id| id.into())
  }

  fn generate_id(&mut self, store: &db::Store) -> db::Result<()> {
    self.id = Some(store.generate_id()?);
    Ok(())
  }
}