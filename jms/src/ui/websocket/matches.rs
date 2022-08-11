use anyhow::bail;
use jms_macros::define_websocket_msg;

use crate::{
  db, models,
  schedule::{playoffs::PlayoffMatchGenerator, quals::QualsMatchGenerator, worker::{SerialisedMatchGeneration, MatchGenerator, SharedMatchGenerators}},
};

use super::{ws::{WebsocketHandler, Websocket, WebsocketContext}, WebsocketMessage2JMS};

define_websocket_msg!($MatchMessage {
  $Quals {
    send Generation(SerialisedMatchGeneration),
    recv Clear,
    recv Generate(<QualsMatchGenerator as MatchGenerator>::ParamType)
  },
  $Playoffs {
    send Generation(SerialisedMatchGeneration),
    recv Clear,
    recv Generate(<PlayoffMatchGenerator as MatchGenerator>::ParamType)
  },
  send All(Vec<models::SerializedMatch>),
  send Next(Option<models::SerializedMatch>),
  send Last(Option<models::SerializedMatch>)
});

pub struct WSMatchHandler(pub SharedMatchGenerators);

#[async_trait::async_trait]
impl WebsocketHandler for WSMatchHandler {
  async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
    {
      let gen = self.0.lock().await;
      ctx.broadcast(MatchMessage2UI::from( MatchMessageQuals2UI::Generation((&gen.quals).into()) ));
      ctx.broadcast(MatchMessage2UI::from( MatchMessagePlayoffs2UI::Generation((&gen.playoffs).into()) ));
    }
    {
      let sorted = models::Match::sorted(&db::database())?;
      ctx.broadcast(MatchMessage2UI::All(sorted.iter().map(|m| m.clone().into()).collect()));
      ctx.broadcast(MatchMessage2UI::Next(sorted.iter().find(|&m| !m.played).map(|m| m.clone().into())));
      ctx.broadcast(MatchMessage2UI::Last(sorted.iter().rev().find(|&m| m.played).map(|m| m.clone().into())));
    }
    Ok(())
  }

  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Match(msg) = msg {
      let gen = self.0.lock().await;
      match msg.clone() {
        MatchMessage2JMS::Quals(msg) => match msg {
          MatchMessageQuals2JMS::Clear if !gen.quals.has_played() => gen.quals.delete(),
          MatchMessageQuals2JMS::Clear => bail!("Cannot clear match generator after matches have started!"),
          MatchMessageQuals2JMS::Generate(p) => gen.quals.generate(p).await,
        },
        MatchMessage2JMS::Playoffs(msg) => match msg {
          MatchMessagePlayoffs2JMS::Clear if !gen.playoffs.has_played() => gen.playoffs.delete(),
          MatchMessagePlayoffs2JMS::Clear => bail!("Cannot clear match generator after matches have started!"),
          MatchMessagePlayoffs2JMS::Generate(p) => gen.playoffs.generate(p).await,
        },
      }

      // Broadcast any new changes
      drop(gen);
      self.broadcast(&ws.context).await?;
    }
    Ok(())
  }
}
