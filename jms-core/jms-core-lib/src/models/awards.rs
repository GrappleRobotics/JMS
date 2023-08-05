use std::convert::Infallible;

use crate::db;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AwardRecipient {
  pub team: Option<String>,
  pub awardee: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Award {
  pub id: String,
  pub name: String,
  pub recipients: Vec<AwardRecipient>,
}

#[async_trait::async_trait]
impl db::Table for Award {
  const PREFIX: &'static str = "db:award";
  type Id = String;
  type Err = Infallible;

  fn id(&self) -> String {
    self.id.clone()
  }
}