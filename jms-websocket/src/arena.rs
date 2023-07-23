use jms_arena_lib::{ArenaState, ArenaSignal, AllianceStation, ArenaRPCClient, ARENA_STATE_KEY};
use jms_core_lib::models::AllianceStationId;
use jms_macros::define_websocket_msg;

use crate::{ws::{WebsocketHandler, WebsocketContext, Websocket}, WebsocketMessage2JMS};

define_websocket_msg!($ArenaMessage {
  $State {
    send Current(ArenaState),
    recv Signal(ArenaSignal, String)
  },
  $Alliance {
    send CurrentStations(Vec<AllianceStation>),
    recv UpdateAlliance {
      station: AllianceStationId,
      bypass: Option<bool>,
      team: Option<usize>,
      estop: Option<bool>,
      astop: Option<bool>
    }
  },
  // $Match {
  //   send Current(Option<LoadedMatch>),
  //   send Score(MatchScoreSnapshot),
  //   recv LoadTest,
  //   recv Unload,
  //   recv Load(String),
  //   recv ScoreUpdate(ScoreUpdateData)
  // },
  // $AudienceDisplay {
  //   send Current(AudienceDisplay),
  //   recv $Set {
  //     Field,
  //     MatchPreview,
  //     MatchPlay,
  //     MatchResults(Option<String>),
  //     AllianceSelection,
  //     PlayoffBracket,
  //     Award(usize),
  //     CustomMessage(String)
  //   },
  //   PlaySound(String)
  // }
});

pub struct WSArenaHandler();

#[async_trait::async_trait]
impl WebsocketHandler for WSArenaHandler {
  async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
    // ctx.broadcast::<ArenaMessage2UI>(ArenaMessageMatch2UI::Current(arena.current_match().await).into()).await;
    // ctx.broadcast::<ArenaMessage2UI>(ArenaMessageMatch2UI::Score(arena.score().await.into()).into()).await;
    ctx.broadcast::<ArenaMessage2UI>(ArenaMessageState2UI::Current(ctx.kv.json_get(ARENA_STATE_KEY, "$").await?).into()).await;
    let mut stations: Vec<AllianceStation> = vec![];
    for stn in AllianceStationId::all() {
      stations.push(ctx.kv.json_get(&stn.to_kv_key(), "$").await?);
    }
    ctx.broadcast::<ArenaMessage2UI>(ArenaMessageAlliance2UI::CurrentStations(stations).into()).await;
    // ctx.broadcast::<ArenaMessage2UI>(ArenaMessageAudienceDisplay2UI::Current(arena.audience().await).into()).await;
    Ok(())
  }

  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Arena(msg) = msg {
      match msg.clone() {
        ArenaMessage2JMS::State(msg) => match msg {
          ArenaMessageState2JMS::Signal(signal, source) => {
            ArenaRPCClient::signal(&ws.context.mq, signal, source).await?.map_err(|x| anyhow::anyhow!(x))?;
          },
        },
        ArenaMessage2JMS::Alliance(msg) => match msg {
          ArenaMessageAlliance2JMS::UpdateAlliance { station, bypass, team, estop, astop } => {
            let current_state: ArenaState = ws.context.kv.json_get(ARENA_STATE_KEY, "$").await?;
            let idle = matches!(current_state, ArenaState::Idle { .. });
            let prestart = matches!(current_state, ArenaState::Prestart { .. });

            match bypass {
              Some(byp) if (idle || prestart) => ws.context.kv.json_set(&station.to_kv_key(), "$.bypass", &byp).await?,
              Some(_) => anyhow::bail!("Can't bypass unless in IDLE or PRESTART"),
              None => ()
            }

            match team {
              Some(0) if idle => ws.context.kv.json_set(&station.to_kv_key(), "$.team", &Option::<u16>::None).await?,
              Some(id) if idle => ws.context.kv.json_set(&station.to_kv_key(), "$.team", &Some(id)).await?,
              Some(_) => anyhow::bail!("Can't set team unless in IDLE"),
              None => ()
            }

            if Some(true) == estop {
              ws.context.kv.json_set(&station.to_kv_key(), "$.estop", &true).await?;
            }

            if Some(true) == astop {
              ws.context.kv.json_set(&station.to_kv_key(), "$.astop", &true).await?;
            }
          },
        },
        // ArenaMessage2JMS::Match(msg) => match msg {
        //   ArenaMessageMatch2JMS::LoadTest => arena.arena_impl().load_match(LoadedMatch::new(models::Match::new_test())).await?,
        //   ArenaMessageMatch2JMS::Unload => arena.arena_impl().unload_match().await?,
        //   ArenaMessageMatch2JMS::Load(match_id) => arena.arena_impl().load_match(LoadedMatch::new(models::Match::get_or_err(match_id, &db::database())?)).await?,
        //   ArenaMessageMatch2JMS::ScoreUpdate(update) => {
        //     let a = arena.arena_impl();
        //     let mut score = a.score.write().await;
        //     match update.alliance {
        //       models::Alliance::Blue => score.blue.update(update.update),
        //       models::Alliance::Red => score.red.update(update.update),
        //     }
        //   },
        // },
        // ArenaMessage2JMS::AudienceDisplay(msg) => match msg {
        //   ArenaMessageAudienceDisplay2JMS::Set(set_msg) => {
        //     *(arena.arena_impl().audience.write().await) = match set_msg {
        //       ArenaMessageAudienceDisplaySet2JMS::Field => AudienceDisplay::Field,
        //       ArenaMessageAudienceDisplaySet2JMS::MatchPreview => AudienceDisplay::MatchPreview,
        //       ArenaMessageAudienceDisplaySet2JMS::MatchPlay => AudienceDisplay::MatchPlay,
        //       ArenaMessageAudienceDisplaySet2JMS::MatchResults(match_id) => match match_id {
        //         Some(match_id) => AudienceDisplay::MatchResults(models::Match::get_or_err(match_id, &db::database())?.into()),
        //         None => {
        //           let last_match = models::Match::sorted(&db::database())?.iter().filter(|&t| t.played).last().cloned();
        //           match last_match {
        //             Some(m) => AudienceDisplay::MatchResults(m.into()),
        //             None => bail!("Can't display results when no matches have been played!"),
        //           }
        //         },
        //       },
        //       ArenaMessageAudienceDisplaySet2JMS::AllianceSelection => AudienceDisplay::AllianceSelection,
        //       ArenaMessageAudienceDisplaySet2JMS::PlayoffBracket => AudienceDisplay::PlayoffBracket,
        //       ArenaMessageAudienceDisplaySet2JMS::Award(award_id) => AudienceDisplay::Award(models::Award::get_or_err(award_id, &db::database())?),
        //       ArenaMessageAudienceDisplaySet2JMS::CustomMessage(custom_msg) => AudienceDisplay::CustomMessage(custom_msg),
        //     }
        //   },
        //   ArenaMessageAudienceDisplay2JMS::PlaySound(sound) => {
        //     ws.context.broadcast::<ArenaMessage2UI>(ArenaMessageAudienceDisplay2UI::PlaySound(sound).into()).await;
        //   }
        // }
      }

      // Broadcast when there's any changes
      // self.broadcast(&ws.context).await?;
    }

    Ok(())
  }
}
