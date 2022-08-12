use jms_macros::define_websocket_msg;

use crate::{models::{MatchStationStatusRecordKey, MatchStationStatusRecord}, db::{self, TableType}};

use super::{ws::{WebsocketHandler, Websocket, WebsocketContext}, WebsocketMessage2JMS};

define_websocket_msg!($HistorianMessage {
  send Keys(Vec<MatchStationStatusRecordKey>),
  recv Load(Vec<MatchStationStatusRecordKey>),
  send Load(Vec<MatchStationStatusRecord>)
});

pub struct WSHistorianHandler;

#[async_trait::async_trait]
impl WebsocketHandler for WSHistorianHandler {
  async fn broadcast(&self, ctx: &WebsocketContext) -> anyhow::Result<()> {
    {
      let keys = MatchStationStatusRecord::keys(&db::database())?;
      ctx.broadcast::<HistorianMessage2UI>(HistorianMessage2UI::Keys(keys.into_iter().map(|x| x.0).collect()).into());
    }
    Ok(())
  }
  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Historian(msg) = msg {
      match msg.clone() {
        HistorianMessage2JMS::Load(keys) => {
          let records = MatchStationStatusRecord::get_all(keys.into_iter(), &db::database())?;
          ws.reply(HistorianMessage2UI::Load(records.into_iter().flat_map(|x| x).collect())).await;
        },
      }
    }

    Ok(())
  }
}