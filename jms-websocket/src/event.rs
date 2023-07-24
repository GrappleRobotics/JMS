use jms_core_lib::{models, db::{DBSingleton, Table, self}};
use jms_macros::define_websocket_msg;

use super::{ws::{WebsocketHandler, Websocket, WebsocketContext}, WebsocketMessage2JMS};

define_websocket_msg!($EventMessage {
  $Details {
    send Current(models::EventDetails),
    recv Update(models::EventDetails)
  },
  $Team {
    send CurrentAll(Vec<models::Team>),
    recv Insert(models::Team),
    recv Delete(String)
  },
  // $Schedule {
  //   send CurrentBlocks(Vec<models::ScheduleBlock>),
  //   recv NewBlock,
  //   recv DeleteBlock(usize),
  //   recv UpdateBlock(models::ScheduleBlock),
  //   recv LoadDefault(usize)   // Time since unix epoch, on the starting day of the event
  // },
  // $Alliance {
  //   send CurrentAll(Vec<models::PlayoffAlliance>),
  //   recv Create(usize),
  //   recv Clear,
  //   recv Update(models::PlayoffAlliance),
  //   recv Promote
  // },
  // // TODO:
  // // $Ranking {
  // //   send CurrentAll(Vec<models::TeamRanking>)
  // // },
  $Award {
    send CurrentAll(Vec<models::Award>),
    recv Create(String),
    recv Update(models::Award),
    recv Delete(String)
  }
});

pub struct WSEventHandler;

#[async_trait::async_trait]
impl WebsocketHandler for WSEventHandler {
  async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
    ctx.broadcast::<EventMessage2UI>(EventMessageDetails2UI::Current( models::EventDetails::get(&ctx.kv)? ).into()).await;
    ctx.broadcast::<EventMessage2UI>(EventMessageTeam2UI::CurrentAll( models::Team::all(&ctx.kv)? ).into()).await;
    // ctx.broadcast::<EventMessage2UI>(EventMessageSchedule2UI::CurrentBlocks( models::ScheduleBlock::sorted(&ctx.kv).await? ).into()).await;
    // ctx.broadcast::<EventMessage2UI>(EventMessageAlliance2UI::CurrentAll( models::PlayoffAlliance::all(&ctx.kv).await? ).into()).await;
    // ctx.broadcast::<EventMessage2UI>(EventMessageRanking2UI::CurrentAll( models::TeamRanking::sorted(&db::database())? ).into()).await;
    ctx.broadcast::<EventMessage2UI>(EventMessageAward2UI::CurrentAll( models::Award::all(&ctx.kv)? ).into()).await;
    Ok(())
  }

  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Event(msg) = msg {
      match msg.clone() {
        EventMessage2JMS::Details(msg) => match msg {
          EventMessageDetails2JMS::Update(mut details) => { details.update(&ws.context.kv)?; },
        },
        EventMessage2JMS::Team(msg) => match msg {
            EventMessageTeam2JMS::Insert(team) => { team.maybe_gen_wpa().insert(&ws.context.kv)?; },
            EventMessageTeam2JMS::Delete(team_id) => { models::Team::delete_by(&team_id, &ws.context.kv)?; },
        },
        // EventMessage2JMS::Schedule(msg) => match msg {
        //     EventMessageSchedule2JMS::NewBlock => { models::ScheduleBlock::append_default(&db::database())?; },
        //     EventMessageSchedule2JMS::DeleteBlock(block_id) => { models::ScheduleBlock::remove_by(block_id, &db::database())?; },
        //     EventMessageSchedule2JMS::UpdateBlock(mut block) => { block.insert(&db::database())?; },
        //     EventMessageSchedule2JMS::LoadDefault(timestamp) => { 
        //       let start_day = Local.from_utc_datetime(&NaiveDateTime::from_timestamp((timestamp).try_into()?, 0)).date();
        //       models::ScheduleBlock::generate_default_2day(start_day, &db::database())?;
        //     },
        // },
        // EventMessage2JMS::Alliance(msg) => match msg {
        //     EventMessageAlliance2JMS::Create(count) => { models::PlayoffAlliance::create_all(count, &db::database())?; },
        //     EventMessageAlliance2JMS::Clear => { models::PlayoffAlliance::clear(&db::database())?; },
        //     EventMessageAlliance2JMS::Update(mut alliance) => { alliance.insert(&db::database())?; },
        //     EventMessageAlliance2JMS::Promote => { models::PlayoffAlliance::promote(&db::database())?; },
        // },
        EventMessage2JMS::Award(msg) => match msg {
            EventMessageAward2JMS::Create(name) => { models::Award { id: db::generate_id(), name: name.clone(), recipients: vec![] }.insert(&ws.context.kv)?; },
            EventMessageAward2JMS::Update(mut award) => { award.insert(&ws.context.kv)?; },
            EventMessageAward2JMS::Delete(award_id) => { models::Award::delete_by(&award_id, &ws.context.kv)?; },
        }
      };

      // Broadcast when there's any changes
      self.broadcast(&ws.context).await?;
    }
    Ok(())
  }
}