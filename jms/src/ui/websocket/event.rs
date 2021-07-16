use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods};

use crate::{db, models};

use super::WebsocketMessageHandler;

pub struct EventWebsocketHandler;

#[async_trait::async_trait]
impl WebsocketMessageHandler for EventWebsocketHandler {
  async fn handle(&mut self, msg: super::JsonMessage) -> super::Result<Option<super::JsonMessage>> {
    let response_msg = msg.response();
    
    Ok(match msg.noun.as_str() {
      "details" => match (msg.verb.as_str(), msg.data) {
        ("get", None) => {
          let ed = models::EventDetails::get(&db::connection())?;
          Some(response_msg.data(serde_json::to_value(ed)?))
        },
        ("update", Some(data)) => {
          use crate::schema::event_details::dsl::*;
          let ed: models::EventDetails = serde_json::from_value(data)?;
          diesel::update(event_details).set(&ed).execute(&db::connection())?;
          None
        },
        _ => Some(response_msg.invalid_verb_or_data())
      },
      "teams" => match (msg.verb.as_str(), msg.data) {
        ("get", None) => {
          use crate::schema::teams::dsl::*;
          let ts = teams.load::<models::Team>(&db::connection())?;
          Some(response_msg.data(serde_json::to_value(ts)?))
        },
        ("insert", Some(data)) => {
          use crate::schema::teams::dsl::*;
          let team: models::Team = serde_json::from_value(data)?;
          // TODO: Validate
          diesel::replace_into(teams)
            .values(&team)
            .execute(&db::connection())?;
          None
        },
        ("delete", Some(serde_json::Value::Number(team_id))) => {
          use crate::schema::teams::dsl::*;
          if let Some(n) = team_id.as_i64() {
            diesel::delete(teams.filter(id.eq(n as i32))).execute(&db::connection())?;
            None
          } else {
            Some(response_msg.invalid_verb_or_data())
          }
        }
        _ => Some(response_msg.invalid_verb_or_data())
      },
      _ => Some(response_msg.unknown_noun())
    })
  }
}