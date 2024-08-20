use crate::db::Singleton;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EventDetails {
  pub code: Option<String>,
  pub event_name: Option<String>,
  pub webcasts: Vec<String>,
  pub av_chroma_key: String,
  pub av_event_colour: String,
  pub event_logo: Option<String>    // Encoded as a HTML5 img src
}

#[async_trait::async_trait]
impl Singleton for EventDetails {
  const KEY: &'static str = "db:event_details";
}

impl Default for EventDetails {
  fn default() -> Self {
    Self { code: None, event_name: None, webcasts: vec![], av_chroma_key: "#f0f".to_owned(), av_event_colour: "#e9ab01".to_owned(), event_logo: None }
  }
}