use jms_core_lib::{models, db::Table};
use jms_macros::define_websocket_msg;

use super::{ws::{WebsocketHandler, Websocket, WebsocketContext}, WebsocketMessage2JMS, WebsocketMessage2UI};

define_websocket_msg!($MatchMessage {
  // $Quals {
  //   send Generation(SerialisedMatchGeneration),
  //   recv Clear,
  //   recv Generate(<QualsMatchGenerator as MatchGenerator>::ParamType)
  // },
  // $Playoffs {
  //   send Generation(SerialisedMatchGeneration),
  //   recv Clear,
  //   recv Generate(<PlayoffMatchGenerator as MatchGenerator>::ParamType)
  // },
  recv Reset(String),
  recv Delete(String),
  recv Update(models::Match),
  send All(Vec<models::SerializedMatch>),
  send Next(Option<models::SerializedMatch>),
  send Last(Option<models::SerializedMatch>)
});

pub struct WSMatchHandler();

#[async_trait::async_trait]
impl WebsocketHandler for WSMatchHandler {
  async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
    // {
    //   let gen = self.0.lock().await;
    //   ctx.broadcast(MatchMessage2UI::from( MatchMessageQuals2UI::Generation((&gen.quals).into()) )).await;
    //   ctx.broadcast(MatchMessage2UI::from( MatchMessagePlayoffs2UI::Generation((&gen.playoffs).into()) )).await;
    // }
    {
      let sorted = models::Match::all(&ctx.kv)?;
      ctx.broadcast(MatchMessage2UI::All(sorted.iter().map(|m| m.clone().into()).collect())).await;
      ctx.broadcast(MatchMessage2UI::Next(sorted.iter().find(|&m| !m.played).map(|m| m.clone().into()))).await;
      ctx.broadcast(MatchMessage2UI::Last(sorted.iter().rev().find(|&m| m.played).map(|m| m.clone().into()))).await;
    }
    Ok(())
  }

  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Match(msg) = msg {
      match msg.clone() {
        // MatchMessage2JMS::Quals(msg) => match msg {
        //   MatchMessageQuals2JMS::Clear if !gen.quals.has_played() => gen.quals.delete(),
        //   MatchMessageQuals2JMS::Clear => bail!("Cannot clear match generator after matches have started!"),
        //   MatchMessageQuals2JMS::Generate(p) => gen.quals.generate(p).await,
        // },
        // MatchMessage2JMS::Playoffs(msg) => match msg {
        //   MatchMessagePlayoffs2JMS::Clear if !gen.playoffs.has_played() => gen.playoffs.delete(),
        //   MatchMessagePlayoffs2JMS::Clear => bail!("Cannot clear match generator after matches have started!"),
        //   MatchMessagePlayoffs2JMS::Generate(p) => gen.playoffs.generate(p).await,
        // },
        MatchMessage2JMS::Update(m) => {
          ws.require_fta().await?;
          m.insert(&ws.context.kv)?;
        }
        MatchMessage2JMS::Delete(id) => {
          ws.require_fta().await?;
          match models::Match::delete_by(&id, &ws.context.kv) {
            Ok(_) => (),
            Err(_) => { ws.reply(WebsocketMessage2UI::Error("No match with given ID".to_owned())).await; }
          }
        },
        MatchMessage2JMS::Reset(id) => {
          ws.require_fta().await?;
          if let Ok(mut m) = models::Match::get(&id, &ws.context.kv) {
            m.reset();
            m.insert(&ws.context.kv)?;
          }
        }
      }

      // Broadcast any new changes
      self.broadcast(&ws.context).await?;
    }
    Ok(())
  }
}
