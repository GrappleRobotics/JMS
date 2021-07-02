use log::info;
use serde_json::json;

use super::WebsocketMessageHandler;

pub struct ArenaWebsocketHandler;

impl WebsocketMessageHandler for ArenaWebsocketHandler {
  fn handle(&mut self, object: String, noun: String, verb: String, data: Option<serde_json::Value>) -> super::Result<Option<super::JsonMessage>> {
    info!("{} {} -> {:?}", noun, verb, data);
    if noun == "mode" {
      return Ok(Some(super::JsonMessage {
        object, noun, verb,
        data: Some(json!({ "name": "Jaci", "age": 22 }))
      }));
    }
    Ok(None)
  }
}