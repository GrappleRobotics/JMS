use crate::db;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AwardRecipient {
  pub team: Option<usize>,
  pub awardee: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Award {
  pub id: Option<usize>,
  pub name: String,
  pub recipients: Vec<AwardRecipient>,
}

impl db::TableType for Award {
  const TABLE: &'static str = "awards";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    self.id.map(|id| id.into())
  }

  fn set_id(&mut self, id: Self::Id) {
    self.id = Some(id.into())
  }
}