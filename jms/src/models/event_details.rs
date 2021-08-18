use crate::db::{self, TableType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventDetails {
  pub code: Option<String>,
  pub event_name: Option<String>,
}

impl db::TableType for EventDetails {
  const TABLE: &'static str = "event_details";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    Some(1.into())
  }
}

impl EventDetails {
  pub fn get(store: &db::Store) -> db::Result<EventDetails> {
    let first = Self::table(store)?.first()?;

    match first {
      Some(ed) => Ok(ed),
      None => {
        let ed = EventDetails { code: None, event_name: None };
        ed.insert(store)?;
        Ok(ed)
      },
    }
  }
}
