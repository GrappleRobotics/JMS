use crate::db::{DBDateTime, self};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SupportTicket {
  pub id: Option<u64>,
  pub team: usize,
  pub match_id: Option<String>,
  pub issue_type: String,
  pub author: String,
  pub notes: Vec<TicketComment>,
  pub assigned_to: Option<String>,
  pub resolved: bool
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TicketComment {
  pub author: String,
  pub time: DBDateTime,
  pub comment: String
}

impl db::TableType for SupportTicket {
  const TABLE: &'static str = "tickets";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    self.id.map(Into::into)
  }

  fn generate_id(&mut self, d: &db::Store) -> db::Result<()> {
    self.id = Some(d.generate_id()?);
    Ok(())
  }
}