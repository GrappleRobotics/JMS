use chrono::{Duration, Local, NaiveTime, TimeZone};
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods};

use crate::{db, models::{self, SQLDuration}};

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
          let sbs = schedule_blocks.load::<models::ScheduleBlock>(&db::connection())?;
          Some(response_msg.data(serde_json::to_value(sbs)?))
        },
        ("new_block", None) => {
          // TODO: Validate, can't be done if the schedule is locked
          use crate::schema::schedule_blocks::dsl::*;
          let mut start = Local::today().and_hms(9, 00, 00);

          match schedule_blocks.order(id.desc()).first::<models::ScheduleBlock>(&db::connection()) {
            Ok(sb) => {
              let end = Local.from_local_datetime(&sb.end_time).unwrap();
              let new_start = end + Duration::hours(1);
              let new_end = new_start + Duration::hours(3);

              if new_end.time() >= NaiveTime::from_hms(17, 00, 00) {
                // Automatically move to tomorrow
                start = (end + Duration::days(1)).date().and_hms(9, 00, 00);
              } else {
                start = new_start;
              }
            },
            Err(diesel::NotFound) => (),
            Err(e) => return Err(e.into()),
          }

          diesel::insert_into(schedule_blocks)
            .values((
              name.eq("Unnamed Block"),
              start_time.eq(start.naive_local()),
              end_time.eq((start + Duration::hours(3)).naive_local()),
              cycle_time.eq(SQLDuration(Duration::minutes(13)))
            ))
            .execute(&db::connection())?;
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
        }
        _ => Some(response_msg.invalid_verb_or_data())
      }
      _ => Some(response_msg.unknown_noun())
    })
  }
}