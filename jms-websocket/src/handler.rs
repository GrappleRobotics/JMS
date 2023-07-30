use jms_core_lib::models::MaybeToken;
use schemars::{gen::SchemaGenerator, schema::Schema};

use crate::ws::WebsocketContext;

#[async_trait::async_trait]
pub trait WebsocketHandler {
  async fn update_publishers(&self, context: &WebsocketContext) -> anyhow::Result<Vec<(String, serde_json::Value)>>;
  async fn on_subscribe(&self, topic: &str) -> anyhow::Result<Vec<(String, serde_json::Value)>>;
  async fn process_rpc_call(&self, ctx: &WebsocketContext, token: &MaybeToken, path: String, msg: Option<serde_json::Value>) -> anyhow::Result<(String, serde_json::Value)>;

  fn publishers(&self) -> Vec<String>;
  fn rpcs(&self) -> Vec<String>;

  // fn publish_schema(&self, gen: &mut SchemaGenerator) -> Schema;
  // fn rpc_request_schema(&self, gen: &mut SchemaGenerator) -> Schema;
  // fn rpc_response_schema(&self, gen: &mut SchemaGenerator) -> Schema;
}