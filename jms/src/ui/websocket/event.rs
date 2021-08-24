use anyhow::bail;

use chrono::{Local, NaiveDateTime, TimeZone};

use crate::{db::{self, TableType}, models::{self, PlayoffAlliance, ScheduleBlock}};

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
      let ed = models::EventDetails::get(&db::database())?;
      response.push(msg.noun("details").to_data(&ed)?)
    }
    {
      // Teams
      let ts = models::Team::all(&db::database())?;
      response.push(msg.noun("teams").to_data(&ts)?)
    }
    {
      // Schedule
      let sbs = models::ScheduleBlock::sorted(&db::database())?;
      response.push(msg.noun("schedule").to_data(&sbs)?)
    }
    {
      // Alliances
      let als = models::PlayoffAlliance::all(&db::database())?;
      response.push(msg.noun("alliances").to_data(&als)?)
    }
    {
      // Rankings
      let rs = models::TeamRanking::sorted(&db::database())?;
      response.push(msg.noun("rankings").to_data(&rs)?)
    }
    {
      // Awards
      let aws = models::Award::all(&db::database())?;
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
          let mut ed: models::EventDetails = serde_json::from_value(data)?;
          ed.insert(&db::database())?;
        }
        _ => bail!("Invalid verb or data"),
      },
      "teams" => match (msg.verb.as_str(), msg.data) {
        ("insert", Some(data)) => {
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
          team.insert(&db::database())?;
        }
        ("delete", Some(serde_json::Value::Number(team_id))) => {
          if let Some(n) = team_id.as_u64() {
            models::Team::remove_by(n, &db::database())?;
          } else {
            bail!("Not a u64: {}", team_id);
          }
        }
        _ => bail!("Invalid verb or data"),
      },
      "schedule" => match (msg.verb.as_str(), msg.data) {
        ("new_block", None) => {
          // TODO: Validate, can't be done if the schedule is locked
          ScheduleBlock::append_default(&db::database())?;
        }
        ("delete_block", Some(serde_json::Value::Number(block_id))) => {
          if let Some(n) = block_id.as_u64() {
            models::ScheduleBlock::remove_by(n, &db::database())?;
          } else {
            bail!("Not a u64: {}", block_id);
          }
        }
        ("update_block", Some(data)) => {
          let mut block: models::ScheduleBlock = serde_json::from_value(data)?;
          block.insert(&db::database())?;
        }
        ("load_default", Some(serde_json::Value::Number(data))) => {
          if let Some(n) = data.as_i64() {
            let date = Local.from_utc_datetime(&NaiveDateTime::from_timestamp(n, 0)).date();
            ScheduleBlock::generate_default_2day(date, &db::database())?;
          } else {
            bail!("Not an i64: {}", data);
          }
        }
        _ => bail!("Invalid verb or data"),
      },
      "alliances" => match (msg.verb.as_str(), msg.data) {
        ("create", Some(serde_json::Value::Number(n))) => {
          if let Some(n) = n.as_u64() {
            PlayoffAlliance::create_all(n as usize, &db::database())?;
          } else {
            bail!("Not a u64: {}", n);
          }
        }
        ("clear", None) => {
          PlayoffAlliance::clear(&db::database())?;
        }
        ("update", Some(data)) => {
          let mut alliance: models::PlayoffAlliance = serde_json::from_value(data)?;
          alliance.insert(&db::database())?;
        }
        ("promote", None) => {
          PlayoffAlliance::promote(&db::database())?;
        }
        _ => bail!("Invalid verb or data"),
      },
      "awards" => match (msg.verb.as_str(), msg.data) {
        ("create", Some(serde_json::Value::String(s))) => {
          let mut award = models::Award {
            id: None,
            name: s,
            recipients: vec![]
          };
          award.insert(&db::database())?;
        }
        ("update", Some(data)) => {
          let mut award: models::Award = serde_json::from_value(data)?;
          award.insert(&db::database())?;
        }
        ("delete", Some(serde_json::Value::Number(award_id))) => {
          if let Some(n) = award_id.as_u64() {
            models::Award::remove_by(n, &db::database())?;
          } else {
            bail!("Not a u64: {}", award_id);
          }
        }
        _ => bail!("Invalid verb or data"),
      },
      _ => bail!("Unknown noun"),
    };

    Ok(response)
  }
}
