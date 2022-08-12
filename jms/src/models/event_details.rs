use crate::db::{self, TableType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EventDetails {
  pub code: Option<String>,
  pub event_name: Option<String>,
  pub webcasts: Vec<String>,
  pub av_chroma_key: String,
  pub av_event_colour: String
}

impl db::TableType for EventDetails {
  const TABLE: &'static str = "event_details";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    Some(1.into())
  }

  fn set_id(&mut self, _id: Self::Id) {}
}

impl EventDetails {
  pub fn get(store: &db::Store) -> db::Result<EventDetails> {
    let first = Self::table(store)?.first()?;

    match first {
      Some(ed) => Ok(ed),
      None => {
        let mut ed = EventDetails { code: None, event_name: None, webcasts: vec![], av_chroma_key: "#f0f".to_owned(), av_event_colour: "#e9ab01".to_owned() };
        ed.insert(store)?;
        Ok(ed)
      },
    }
  }
}
