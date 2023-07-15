use schemars::JsonSchema;
use serde::Serialize;

use crate::models;

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(tag = "scene", content = "params")]
pub enum AudienceDisplay {
  Field,
  MatchPreview,
  MatchPlay,
  MatchResults(models::SerializedMatch),
  AllianceSelection,
  PlayoffBracket,
  Award(models::Award),
  CustomMessage(String),
}