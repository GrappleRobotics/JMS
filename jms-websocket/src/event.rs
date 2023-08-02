use jms_core_lib::{models::{EventDetails, MaybeToken, Permission, ScheduleBlock, ScheduleBlockType, ScheduleBlockUpdate}, db::{Singleton, Table}};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait EventWebsocket {
  #[publish]
  async fn details(&self, ctx: &WebsocketContext) -> anyhow::Result<EventDetails> {
    Ok(EventDetails::get(&ctx.kv)?)
  }

  #[endpoint]
  async fn update(&self, ctx: &WebsocketContext, token: &MaybeToken, details: EventDetails) -> anyhow::Result<EventDetails> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageEvent])?;
    details.update(&ctx.kv)?;
    Ok(details)
  }

  // Schedule

  #[endpoint]
  async fn schedule_get(&self, ctx: &WebsocketContext, _token: &MaybeToken) -> anyhow::Result<Vec<ScheduleBlock>> {
    Ok(ScheduleBlock::all(&ctx.kv)?)
  }

  #[endpoint]
  async fn schedule_new_block(&self, ctx: &WebsocketContext, token: &MaybeToken, block_type: ScheduleBlockType, name: String, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> anyhow::Result<ScheduleBlock> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageSchedule])?;
    let block = ScheduleBlock::new(block_type, name, start, end);
    block.insert(&ctx.kv)?;
    Ok(block)
  }

  #[endpoint]
  async fn schedule_delete(&self, ctx: &WebsocketContext, token: &MaybeToken, block_id: String) -> anyhow::Result<()> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageSchedule])?;
    ScheduleBlock::delete_by(&block_id, &ctx.kv)?;
    Ok(())
  }

  #[endpoint]
  async fn schedule_edit(&self, ctx: &WebsocketContext, token: &MaybeToken, block_id: String, updates: Vec<ScheduleBlockUpdate>) -> anyhow::Result<ScheduleBlock> {
    token.auth(&ctx.kv)?.require_permission(&[Permission::ManageSchedule])?;
    let mut block = ScheduleBlock::get(&block_id, &ctx.kv)?;
    for update in updates {
      update.apply(&mut block);
    }
    block.insert(&ctx.kv)?;
    Ok(block)
  }
}

// use super::{ws::{WebsocketHandler, Websocket, WebsocketContext}, WebsocketMessage2JMS};

// define_websocket_msg!($EventMessage {
//   $Details {
//     send Current(models::EventDetails),
//     recv Update(models::EventDetails)
//   },
//   $Team {
//     send CurrentAll(Vec<models::Team>),
//     recv Insert(models::Team),
//     recv Delete(String)
//   },
//   $Schedule {
//     send CurrentBlocks(Vec<models::ScheduleBlock>),
//     recv NewBlock,
//     recv DeleteBlock(String),
//     recv UpdateBlock(models::ScheduleBlock),
//     recv LoadDefault(usize)   // Time since unix epoch, on the starting day of the event
//   },
//   // $Alliance {
//   //   send CurrentAll(Vec<models::PlayoffAlliance>),
//   //   recv Create(usize),
//   //   recv Clear,
//   //   recv Update(models::PlayoffAlliance),
//   //   recv Promote
//   // },
//   // // TODO:
//   // // $Ranking {
//   // //   send CurrentAll(Vec<models::TeamRanking>)
//   // // },
//   $Award {
//     send CurrentAll(Vec<models::Award>),
//     recv Create(String),
//     recv Update(models::Award),
//     recv Delete(String)
//   }
// });

// pub struct WSEventHandler;

// #[async_trait::async_trait]
// impl WebsocketHandler for WSEventHandler {
//   async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
//     ctx.broadcast::<EventMessage2UI>(EventMessageDetails2UI::Current( models::EventDetails::get(&ctx.kv)? ).into()).await;
//     ctx.broadcast::<EventMessage2UI>(EventMessageTeam2UI::CurrentAll( models::Team::all(&ctx.kv)? ).into()).await;
//     ctx.broadcast::<EventMessage2UI>(EventMessageSchedule2UI::CurrentBlocks( models::ScheduleBlock::sorted(&ctx.kv)? ).into()).await;
//     // ctx.broadcast::<EventMessage2UI>(EventMessageAlliance2UI::CurrentAll( models::PlayoffAlliance::all(&ctx.kv).await? ).into()).await;
//     // ctx.broadcast::<EventMessage2UI>(EventMessageRanking2UI::CurrentAll( models::TeamRanking::sorted(&db::database())? ).into()).await;
//     ctx.broadcast::<EventMessage2UI>(EventMessageAward2UI::CurrentAll( models::Award::all(&ctx.kv)? ).into()).await;
//     Ok(())
//   }

//   async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
//     if let WebsocketMessage2JMS::Event(msg) = msg {
//       match msg.clone() {
//         EventMessage2JMS::Details(msg) => match msg {
//           EventMessageDetails2JMS::Update(details) => { details.update(&ws.context.kv)?; },
//         },
//         EventMessage2JMS::Team(msg) => match msg {
//             EventMessageTeam2JMS::Insert(team) => { team.maybe_gen_wpa().insert(&ws.context.kv)?; },
//             EventMessageTeam2JMS::Delete(team_id) => { models::Team::delete_by(&team_id, &ws.context.kv)?; },
//         },
//         EventMessage2JMS::Schedule(msg) => match msg {
//             EventMessageSchedule2JMS::NewBlock => { models::ScheduleBlock::append_default(&ws.context.kv)?; },
//             EventMessageSchedule2JMS::DeleteBlock(block_id) => { models::ScheduleBlock::delete_by(&block_id, &ws.context.kv)?; },
//             EventMessageSchedule2JMS::UpdateBlock(block) => { block.insert(&ws.context.kv)?; },
//             EventMessageSchedule2JMS::LoadDefault(timestamp) => { 
//               let start_day = chrono::Local.from_utc_datetime(&chrono::NaiveDateTime::from_timestamp((timestamp).try_into()?, 0)).date();
//               models::ScheduleBlock::generate_default_2day(start_day, &ws.context.kv)?;
//             },
//         },
//         // EventMessage2JMS::Alliance(msg) => match msg {
//         //     EventMessageAlliance2JMS::Create(count) => { models::PlayoffAlliance::create_all(count, &db::database())?; },
//         //     EventMessageAlliance2JMS::Clear => { models::PlayoffAlliance::clear(&db::database())?; },
//         //     EventMessageAlliance2JMS::Update(mut alliance) => { alliance.insert(&db::database())?; },
//         //     EventMessageAlliance2JMS::Promote => { models::PlayoffAlliance::promote(&db::database())?; },
//         // },
//         EventMessage2JMS::Award(msg) => match msg {
//             EventMessageAward2JMS::Create(name) => { models::Award { id: db::generate_id(), name: name.clone(), recipients: vec![] }.insert(&ws.context.kv)?; },
//             EventMessageAward2JMS::Update(award) => { award.insert(&ws.context.kv)?; },
//             EventMessageAward2JMS::Delete(award_id) => { models::Award::delete_by(&award_id, &ws.context.kv)?; },
//         }
//       };

//       // Broadcast when there's any changes
//       self.broadcast(&ws.context).await?;
//     }
//     Ok(())
//   }
// }