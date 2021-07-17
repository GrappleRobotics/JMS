use chrono::{Duration, Local, NaiveTime, TimeZone};
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods};

use crate::{db, models::{self, SQLDuration, SQLDatetime, ScheduleBlock}};

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
      "schedule" => match (msg.verb.as_str(), msg.data) {
        ("blocks", None) => {
          use crate::schema::schedule_blocks::dsl::*;
          let sbs = schedule_blocks.order_by(start_time.asc()).load::<models::ScheduleBlock>(&db::connection())?;
          Some(response_msg.data(serde_json::to_value(sbs)?))
        },
        ("new_block", None) => {
          // TODO: Validate, can't be done if the schedule is locked
          ScheduleBlock::append_default(&db::connection())?;
          None
        },
        ("delete_block", Some(serde_json::Value::Number(block_id))) => {
          use crate::schema::schedule_blocks::dsl::*;
          if let Some(n) = block_id.as_i64() {
            diesel::delete(schedule_blocks.filter(id.eq(n as i32))).execute(&db::connection())?;
            None
          } else {
            Some(response_msg.invalid_verb_or_data())
          }
        },
        ("update_block", Some(data)) => {
          use crate::schema::schedule_blocks::dsl::*;
          let block: models::ScheduleBlock = serde_json::from_value(data)?;
          diesel::replace_into(schedule_blocks)
            .values(&block)
            .execute(&db::connection())?;
          None
        },
        ("load_default", None) => {
          ScheduleBlock::generate_default_2day(&db::connection())?;
          None
        }
        _ => Some(response_msg.invalid_verb_or_data())
      }
      _ => Some(response_msg.unknown_noun())
    })
  }
}