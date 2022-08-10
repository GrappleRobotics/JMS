use chrono::{Local, TimeZone, NaiveDateTime};
use jms_macros::define_websocket_msg;

use crate::{db::{self, TableType}, models};

define_websocket_msg!($EventMessage {
  $Details {
    send Current(models::EventDetails),
    recv Update(models::EventDetails)
  },
  $Team {
    send CurrentAll(Vec<models::Team>),
    recv Insert(models::Team),
    recv Delete(usize)
  },
  $Schedule {
    send CurrentBlocks(Vec<models::ScheduleBlock>),
    recv NewBlock,
    recv DeleteBlock(usize),
    recv UpdateBlock(models::ScheduleBlock),
    recv LoadDefault(usize)   // Time since unix epoch, on the starting day of the event
  },
  $Alliance {
    send CurrentAll(Vec<models::PlayoffAlliance>),
    recv Create(usize),
    recv Clear,
    recv Update(models::PlayoffAlliance),
    recv Promote
  },
  $Ranking {
    send CurrentAll(Vec<models::TeamRanking>)
  },
  $Award {
    send CurrentAll(Vec<models::Award>),
    recv Create(String),
    recv Update(models::Award),
    recv Delete(usize)
  }
});

pub async fn ws_periodic_event() -> super::Result<Vec<EventMessage2UI>> {
  let mut data: Vec<EventMessage2UI> = vec![];
  {
    // Details
    let ed = models::EventDetails::get(&db::database())?;
    data.push(EventMessageDetails2UI::Current(ed).into())
  } {
    // Teams
    let ts = models::Team::all(&db::database())?;
    data.push(EventMessageTeam2UI::CurrentAll(ts).into())
  } {
    // Schedule
    let sbs = models::ScheduleBlock::sorted(&db::database())?;
    data.push(EventMessageSchedule2UI::CurrentBlocks(sbs).into())
  } {
    // Alliances
    let als = models::PlayoffAlliance::all(&db::database())?;
    data.push(EventMessageAlliance2UI::CurrentAll(als).into())
  } {
    // Rankings
    let rs = models::TeamRanking::sorted(&db::database())?;
    data.push(EventMessageRanking2UI::CurrentAll(rs).into())
  } {
    // Awards
    let aws = models::Award::all(&db::database())?;
    data.push(EventMessageAward2UI::CurrentAll(aws).into())
  }
  return Ok(data);
}

pub async fn ws_recv_event(data: &EventMessage2JMS) -> super::Result<Vec<super::WebsocketMessage2UI>> {
  let responses = vec![];

  match data.clone() {
    EventMessage2JMS::Details(msg) => match msg {
      EventMessageDetails2JMS::Update(mut details) => { details.insert(&db::database())?; },
    },
    EventMessage2JMS::Team(msg) => match msg {
        EventMessageTeam2JMS::Insert(team) => { team.maybe_gen_wpa().insert(&db::database())?; },
        EventMessageTeam2JMS::Delete(team_id) => { models::Team::remove_by(team_id, &db::database())?; },
    },
    EventMessage2JMS::Schedule(msg) => match msg {
        EventMessageSchedule2JMS::NewBlock => { models::ScheduleBlock::append_default(&db::database())?; },
        EventMessageSchedule2JMS::DeleteBlock(block_id) => { models::ScheduleBlock::remove_by(block_id, &db::database())?; },
        EventMessageSchedule2JMS::UpdateBlock(mut block) => { block.insert(&db::database())?; },
        EventMessageSchedule2JMS::LoadDefault(timestamp) => { 
          let start_day = Local.from_utc_datetime(&NaiveDateTime::from_timestamp((timestamp).try_into()?, 0)).date();
          models::ScheduleBlock::generate_default_2day(start_day, &db::database())?;
        },
    },
    EventMessage2JMS::Alliance(msg) => match msg {
        EventMessageAlliance2JMS::Create(count) => { models::PlayoffAlliance::create_all(count, &db::database())?; },
        EventMessageAlliance2JMS::Clear => { models::PlayoffAlliance::clear(&db::database())?; },
        EventMessageAlliance2JMS::Update(mut alliance) => { alliance.insert(&db::database())?; },
        EventMessageAlliance2JMS::Promote => { models::PlayoffAlliance::promote(&db::database())?; },
    },
    EventMessage2JMS::Award(msg) => match msg {
        EventMessageAward2JMS::Create(name) => { models::Award { id: None, name: name.clone(), recipients: vec![] }.insert(&db::database())?; },
        EventMessageAward2JMS::Update(mut award) => { award.insert(&db::database())?; },
        EventMessageAward2JMS::Delete(award_id) => { models::Award::remove_by(award_id, &db::database())?; },
    }
  };

  return Ok(responses);
}