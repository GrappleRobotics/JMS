use jms_macros::define_websocket_msg;

use super::{ws::{WebsocketHandler, Websocket}, WebsocketMessage2JMS};

define_websocket_msg!($DebugMessage {
  ReplyTest(String)
});

pub struct WSDebugHandler;

#[async_trait::async_trait]
impl WebsocketHandler for WSDebugHandler {
  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Debug(msg) = msg {
      ws.require_fta().await?;

      match msg.clone() {
        DebugMessage2JMS::ReplyTest(word) => { ws.reply::<DebugMessage2UI>(DebugMessage2UI::ReplyTest(word)).await }
      }
    }

    Ok(())
  }
}
