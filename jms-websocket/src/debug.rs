use uuid::Uuid;

use crate::ws::WebsocketContext;

#[jms_websocket_macros::websocket_handler]
pub trait DebugWebsocket {
  #[publish]
  async fn test_publish(&self, _: &WebsocketContext) -> anyhow::Result<String> {
    Ok(format!("Hello World {}", Uuid::new_v4()))
  }

  #[endpoint]
  async fn test_endpoint(&self, _: &WebsocketContext, in_text: String) -> anyhow::Result<String> {
    Ok(in_text.to_uppercase())
  }
}