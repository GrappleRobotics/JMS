use crate::ws::WebsocketContext;

#[async_trait::async_trait]
pub trait WebsocketHandler {
  async fn update_publishers(&self, context: &WebsocketContext) -> anyhow::Result<Vec<serde_json::Value>>;
  async fn on_subscribe(&self, topic: &str) -> anyhow::Result<Vec<serde_json::Value>>;
  async fn process_rpc_call(&self, ctx: &WebsocketContext, msg: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}