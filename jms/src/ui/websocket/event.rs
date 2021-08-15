use anyhow::bail;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::{
  db,
  models::{self, AwardRecipient, PlayoffAlliance, SQLJson, ScheduleBlock, TeamRanking},
};

use super::{JsonMessage, WebsocketMessageHandler};

pub struct EventWebsocketHandler {}

impl EventWebsocketHandler {
  pub fn new() -> Self {
    EventWebsocketHandler {}
  }
}

#[async_trait::async_trait]
impl WebsocketMessageHandler for EventWebsocketHandler {
  async fn update(&mut self) -> super::Result<Vec<JsonMessage>> {
    let msg = JsonMessage::update("event", "");
    let mut response = vec![];
    {
      // Details
      let ed = models::EventDetails::get(&db::connection())?;
      response.push(msg.noun("details").to_data(&ed)?)
    }
    {
      // Teams
      use crate::schema::teams::dsl::*;
      let ts = teams.load::<models::Team>(&db::connection())?;
      response.push(msg.noun("teams").to_data(&ts)?)
    }
    {
      // Schedule
      use crate::schema::schedule_blocks::dsl::*;
      let sbs = schedule_blocks
        .order_by(start_time.asc())
        .load::<models::ScheduleBlock>(&db::connection())?;
      response.push(msg.noun("schedule").to_data(&sbs)?)
    }
    {
      // Alliances
      use crate::schema::playoff_alliances::dsl::*;
      let als = playoff_alliances.load::<models::PlayoffAlliance>(&db::connection())?;
      response.push(msg.noun("alliances").to_data(&als)?)
    }
    {
      // Rankings
      let rs = TeamRanking::get_sorted(&db::connection())?;
      response.push(msg.noun("rankings").to_data(&rs)?)
    }
    {
      // Awards
      use crate::schema::awards::dsl::*;
      let aws = awards.load::<models::Award>(&db::connection())?;
      response.push(msg.noun("awards").to_data(&aws)?)
    }
    Ok(response)
  }

  async fn handle(&mut self, msg: super::JsonMessage) -> super::Result<Vec<super::JsonMessage>> {
    // let response_msg = msg.response();

    let response = vec![];

    match msg.noun.as_str() {
      "details" => match (msg.verb.as_str(), msg.data) {
        ("update", Some(data)) => {
          use crate::schema::event_details::dsl::*;
          let ed: models::EventDetails = serde_json::from_value(data)?;
          diesel::update(event_details).set(&ed).execute(&db::connection())?;
        }
        _ => bail!("Invalid verb or data"),
      },
      "teams" => match (msg.verb.as_str(), msg.data) {
        ("insert", Some(data)) => {
          use crate::schema::teams::dsl::*;
          let mut team: models::Team = serde_json::from_value(data)?;

          if team.wpakey.is_none() {
            use rand::distributions::Alphanumeric;
            use rand::{thread_rng, Rng};

            team.wpakey = Some(
              thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect(),
            )
          }

          // TODO: Validate
          diesel::replace_into(teams).values(&team).execute(&db::connection())?;
        }
        ("delete", Some(serde_json::Value::Number(team_id))) => {
          use crate::schema::teams::dsl::*;
          if let Some(n) = team_id.as_i64() {
            diesel::delete(teams.filter(id.eq(n as i32))).execute(&db::connection())?;
          } else {
            bail!("Not an i64: {}", team_id);
          }
        }
        _ => bail!("Invalid verb or data"),
      },
      "schedule" => match (msg.verb.as_str(), msg.data) {
        ("new_block", None) => {
          // TODO: Validate, can't be done if the schedule is locked
          ScheduleBlock::append_default(&db::connection())?;
        }
        ("delete_block", Some(serde_json::Value::Number(block_id))) => {
          use crate::schema::schedule_blocks::dsl::*;
          if let Some(n) = block_id.as_i64() {
            diesel::delete(schedule_blocks.filter(id.eq(n as i32))).execute(&db::connection())?;
          } else {
            bail!("Not an i64: {}", block_id);
          }
        }
        ("update_block", Some(data)) => {
          use crate::schema::schedule_blocks::dsl::*;
          let block: models::ScheduleBlock = serde_json::from_value(data)?;
          diesel::replace_into(schedule_blocks)
            .values(&block)
            .execute(&db::connection())?;
        }
        ("load_default", None) => {
          ScheduleBlock::generate_default_2day(&db::connection())?;
        }
        _ => bail!("Invalid verb or data"),
      },
      "alliances" => match (msg.verb.as_str(), msg.data) {
        ("create", Some(serde_json::Value::Number(n))) => {
          if let Some(n) = n.as_u64() {
            PlayoffAlliance::create_all(n as usize, &db::connection())?;
          } else {
            bail!("Not a u64: {}", n);
          }
        }
        ("clear", None) => {
          PlayoffAlliance::clear(&db::connection())?;
        }
        ("update", Some(data)) => {
          use crate::schema::playoff_alliances::dsl::*;
          let alliance: models::PlayoffAlliance = serde_json::from_value(data)?;
          diesel::replace_into(playoff_alliances)
            .values(&alliance)
            .execute(&db::connection())?;
        }
        ("promote", None) => {
          PlayoffAlliance::promote(&db::connection())?;
        }
        _ => bail!("Invalid verb or data"),
      },
      "awards" => match (msg.verb.as_str(), msg.data) {
        ("create", Some(serde_json::Value::String(s))) => {
          use crate::schema::awards::dsl::*;
          diesel::insert_into(awards)
            .values((name.eq(s), recipients.eq(SQLJson(Vec::<AwardRecipient>::new()))))
            .execute(&db::connection())?;
        }
        ("update", Some(data)) => {
          use crate::schema::awards::dsl::*;
          let award: models::Award = serde_json::from_value(data)?;
          diesel::replace_into(awards).values(&award).execute(&db::connection())?;
        }
        ("delete", Some(serde_json::Value::Number(award_id))) => {
          use crate::schema::awards::dsl::*;
          if let Some(n) = award_id.as_i64() {
            diesel::delete(awards.filter(id.eq(n as i32))).execute(&db::connection())?;
          } else {
            bail!("Not an i64: {}", award_id);
          }
        }
        _ => bail!("Invalid verb or data"),
      },
      _ => bail!("Unknown noun"),
    };

    Ok(response)
  }
}
