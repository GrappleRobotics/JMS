use crate::db::{DBSingleton, self};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EventDetails {
  pub code: Option<String>,
  pub event_name: Option<String>,
  pub webcasts: Vec<String>,
  pub av_chroma_key: String,
  pub av_event_colour: String
}

impl DBSingleton for EventDetails {
  const ID: &'static str = "event_details";

  fn db_default() -> db::Result<Self> {
    Ok(Self { code: None, event_name: None, webcasts: vec![], av_chroma_key: "#f0f".to_owned(), av_event_colour: "#e9ab01".to_owned() })
  }
}