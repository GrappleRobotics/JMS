use jms_core_lib::{models::JmsComponent, db::Table};

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait ComponentWebsocket {
  #[publish]
  async fn components(&self, ctx: &WebsocketContext) -> anyhow::Result<Vec<JmsComponent>> {
    JmsComponent::all(&ctx.kv)
  }
}