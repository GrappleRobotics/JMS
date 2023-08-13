use std::convert::Infallible;

use crate::db::Table;

#[derive(jms_macros::Updateable)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SupportTicket {
  pub id: String,
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
  pub time: chrono::DateTime<chrono::Local>,
  pub comment: String
}

impl Table for SupportTicket {
  const PREFIX: &'static str = "db:tickets";
  type Err = Infallible;
  type Id = String;

  fn id(&self) -> Self::Id {
    self.id.clone()
  }
}
