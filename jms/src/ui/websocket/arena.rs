use anyhow::{anyhow, bail};

use jms_macros::define_websocket_msg;

use crate::{arena::{matches::LoadedMatch, station::AllianceStationId, ArenaSignal, ArenaState, AudienceDisplay, SharedArena, ArenaAccessRestriction, station::SerialisedAllianceStation}, db::{self, TableType}, models, scoring::scores::ScoreUpdateData};

use super::{WebsocketMessage2JMS, ws::{Websocket, WebsocketContext, WebsocketHandler}};

define_websocket_msg!($ArenaMessage {
  $State {
    send Current(ArenaState),
    recv Signal(ArenaSignal)
  },
  $Alliance {
    send CurrentStations(Vec<SerialisedAllianceStation>),
    recv UpdateAlliance {
      station: AllianceStationId,
      bypass: Option<bool>,
      team: Option<usize>,
      estop: Option<bool>,
      astop: Option<bool>
    }
  },
  $Match {
    send Current(Option<LoadedMatch>),
    recv LoadTest,
    recv Unload,
    recv Load(String),
    recv ScoreUpdate(ScoreUpdateData)
  },
  $AudienceDisplay {
    send Current(AudienceDisplay),
    recv $Set {
      Field,
      MatchPreview,
      MatchPlay,
      MatchResults(Option<String>),
      AllianceSelection,
      Award(usize),
      CustomMessage(String)
    }
  },
  $Access {
    send Current(ArenaAccessRestriction),
    recv Set(ArenaAccessRestriction)
  }
});

pub struct WSArenaHandler(pub SharedArena);

#[async_trait::async_trait]
impl WebsocketHandler for WSArenaHandler {
  async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
    let arena = self.0.lock().await;
    ctx.broadcast::<ArenaMessage2UI>(ArenaMessageMatch2UI::Current(arena.current_match.clone()).into());
    ctx.broadcast::<ArenaMessage2UI>(ArenaMessageState2UI::Current(arena.state.state.clone()).into());
    ctx.broadcast::<ArenaMessage2UI>(ArenaMessageAlliance2UI::CurrentStations(arena.stations.iter().map(|x| x.clone().into()).collect()).into());
    ctx.broadcast::<ArenaMessage2UI>(ArenaMessageAudienceDisplay2UI::Current(arena.audience_display.clone()).into());
    ctx.broadcast::<ArenaMessage2UI>(ArenaMessageAccess2UI::Current(arena.access.clone()).into());
    Ok(())
  }

  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Arena(msg) = msg {
      let mut arena = self.0.lock().await;

      match msg.clone() {
        ArenaMessage2JMS::State(msg) => match msg {
          ArenaMessageState2JMS::Signal(signal) => arena.signal(signal).await,
        },
        ArenaMessage2JMS::Alliance(msg) => match msg {
          ArenaMessageAlliance2JMS::UpdateAlliance { station, bypass, team, estop, astop } => {
            let current_state = arena.current_state();
            let idle = matches!(current_state, ArenaState::Idle { .. });
            let prestart = matches!(current_state, ArenaState::Prestart { .. });

            let stn = arena.station_mut(station).ok_or(anyhow!("No alliance station: {}", station))?;
            match bypass {
              Some(byp) if (idle || prestart) => stn.bypass = byp,
              Some(_) => bail!("Can't bypass unless in IDLE or PRESTART"),
              None => ()
            }

            match team {
              Some(0) if idle => stn.reset(),
              Some(id) if idle => { stn.reset(); stn.team = Some(id as u16) },
              Some(_) => bail!("Can't set team unless in IDLE"),
              None => ()
            }

            stn.estop = stn.estop || estop.unwrap_or(false);
            stn.astop = stn.astop || astop.unwrap_or(false);
          },
        },
        ArenaMessage2JMS::Match(msg) => match msg {
          ArenaMessageMatch2JMS::LoadTest => arena.load_match(LoadedMatch::new(models::Match::new_test()))?,
          ArenaMessageMatch2JMS::Unload => arena.unload_match()?,
          ArenaMessageMatch2JMS::Load(match_id) => arena.load_match(LoadedMatch::new(models::Match::get_or_err(match_id, &db::database())?))?,
          ArenaMessageMatch2JMS::ScoreUpdate(update) => {
            match arena.current_match.as_mut() {
              Some(m) => match update.alliance {
                models::Alliance::Blue => m.score.blue.update(update.update),
                models::Alliance::Red => m.score.red.update(update.update),
              },
              None => bail!("Can't update score: no match is running!"),
            }
          },
        },
        ArenaMessage2JMS::AudienceDisplay(msg) => match msg {
          ArenaMessageAudienceDisplay2JMS::Set(set_msg) => {
            arena.audience_display = match set_msg {
              ArenaMessageAudienceDisplaySet2JMS::Field => AudienceDisplay::Field,
              ArenaMessageAudienceDisplaySet2JMS::MatchPreview => AudienceDisplay::MatchPreview,
              ArenaMessageAudienceDisplaySet2JMS::MatchPlay => AudienceDisplay::MatchPlay,
              ArenaMessageAudienceDisplaySet2JMS::MatchResults(match_id) => match match_id {
                Some(match_id) => AudienceDisplay::MatchResults(models::Match::get_or_err(match_id, &db::database())?.into()),
                None => {
                  let last_match = models::Match::sorted(&db::database())?.iter().filter(|&t| t.played).last().cloned();
                  match last_match {
                    Some(m) => AudienceDisplay::MatchResults(m.into()),
                    None => bail!("Can't display results when no matches have been played!"),
                  }
                },
              },
              ArenaMessageAudienceDisplaySet2JMS::AllianceSelection => AudienceDisplay::AllianceSelection,
              ArenaMessageAudienceDisplaySet2JMS::Award(award_id) => AudienceDisplay::Award(models::Award::get_or_err(award_id, &db::database())?),
              ArenaMessageAudienceDisplaySet2JMS::CustomMessage(custom_msg) => AudienceDisplay::CustomMessage(custom_msg),
            }
          }
        },
        ArenaMessage2JMS::Access(msg) => match msg {
          ArenaMessageAccess2JMS::Set(condition) => arena.access = condition,
        },
      }

      // Broadcast when there's any changes
      drop(arena);
      self.broadcast(&ws.context).await?;
    }

    Ok(())
  }
}
