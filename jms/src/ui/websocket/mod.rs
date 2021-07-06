mod handler;

pub use handler::*;

use futures::{lock::Mutex, SinkExt, StreamExt};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, error, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite};

use crate::{arena::exceptions::ArenaError, context};

#[async_trait::async_trait]
pub trait WebsocketMessageHandler {
  async fn handle(&mut self, msg: JsonMessage) -> Result<Option<JsonMessage>>;
}

pub struct Websockets {
  handlers: Arc<Mutex<HashMap<String, Box<dyn WebsocketMessageHandler + Send>>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonMessage {
  pub object: String,
  pub noun: String,
  pub verb: String,
  pub data: Option<Value>,
  pub error: Option<String>,
}

// Basic builder pattern
impl JsonMessage {
  pub fn new(object: &str, noun: &str, verb: &str) -> JsonMessage {
    JsonMessage {
      object: object.to_owned(),
      noun: noun.to_owned(),
      verb: verb.to_owned(),
      data: None,
      error: None,
    }
  }

  pub fn response(&self) -> JsonMessage {
    let mut n = self.clone();
    n.data = None;
    n.error = None;
    n
  }

  // pub fn noun(&self, noun: &str) -> JsonMessage {
  //   let mut n = self.clone();
  //   n.noun = noun.to_owned();
  //   n
  // }

  // pub fn verb(&self, verb: &str) -> JsonMessage {
  //   let mut n = self.clone();
  //   n.verb = verb.to_owned();
  //   n
  // }

  pub fn data(&self, data: Value) -> JsonMessage {
    let mut n = self.clone();
    n.data = Some(data);
    n
  }

  pub fn error(&self, msg: &str) -> JsonMessage {
    let mut n = self.clone();
    n.error = Some(msg.to_owned());
    n
  }

  pub fn unknown_object(&self) -> JsonMessage {
    self.error("Unknown object")
  }

  pub fn unknown_noun(&self) -> JsonMessage {
    self.error("Unknown noun")
  }

  pub fn invalid_verb_or_data(&self) -> JsonMessage {
    self.error("Invalid verb/data")
  }
}

impl Websockets {
  pub fn new() -> Self {
    Websockets {
      handlers: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub async fn register(&mut self, object_key: &'static str, handler: Box<dyn WebsocketMessageHandler + Send>) {
    self.handlers.lock().await.insert(object_key.to_owned(), handler);
  }

  pub async fn begin(&self) -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    info!("WebSocket started...");

    while let Ok((stream, _addr)) = listener.accept().await {
      let h = self.handlers.clone();
      tokio::spawn(async move {
        context!(&format!("Websocket {}", _addr), {
          if let Err(e) = Self::connection_handler(stream, h).await {
            match e {
              WebsocketError::Tungstenite(ref e) => match e {
                tungstenite::Error::ConnectionClosed | tungstenite::Error::Protocol(_) | tungstenite::Error::Utf8 => (),
                err => error!("Tungstenite Error: {}", err),
              },
              err => error!("Error: {}", err),
            }
          }
        })
      });
    }

    Ok(())
  }

  // Can't be a self method as tokio::spawn may outlive the object itself, unless we constrain to be 'static lifetime
  async fn connection_handler(
    stream: TcpStream,
    handlers: Arc<Mutex<HashMap<String, Box<dyn WebsocketMessageHandler + Send>>>>,
  ) -> Result<()> {
    let mut ws = accept_async(stream).await?;

    while let Some(msg) = ws.next().await {
      let msg = msg?;
      match msg {
        tungstenite::Message::Text(msg_str) => {
          let m: JsonMessage = serde_json::from_str(&msg_str)?;
          let mut hs = handlers.lock().await;

          match hs.get_mut(&m.object) {
            Some(h) => {
              let response = h.handle(m.clone()).await;
              match response {
                Ok(Some(ref r)) => {
                  let response_msg = serde_json::to_string(r)?;
                  ws.send(tungstenite::Message::Text(response_msg)).await?;
                }
                Ok(None) => (),
                Err(e) => {
                  error!("WS Error: {}", e);
                  let response_msg = serde_json::to_string(&m.response().error(&e.to_string()))?;
                  ws.send(tungstenite::Message::Text(response_msg)).await?;
                }
              }
            }
            None => {
              warn!("No WS handler for object {}", m.object);
              let response = serde_json::to_string(&JsonMessage::new(&m.object, &m.noun, &m.verb).unknown_object())?;
              ws.send(tungstenite::Message::Text(response)).await?;
            }
          }
        }
        _ => (),
      }
    }

    Ok(())
  }
}

#[derive(Debug)]
pub enum WebsocketError {
  Tungstenite(tungstenite::Error),
  JSON(serde_json::Error),
  IO(std::io::Error),
  Arena(ArenaError),
  Other(String),
}

impl std::fmt::Display for WebsocketError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      WebsocketError::Tungstenite(ref e) => write!(f, "Tungstenite Error: {}", e),
      WebsocketError::JSON(ref e) => write!(f, "JSON Error: {}", e),
      WebsocketError::IO(ref e) => write!(f, "IO Error: {}", e),
      WebsocketError::Arena(ref e) => write!(f, "Arena Error: {}", e),
      WebsocketError::Other(ref s) => write!(f, "Error: {}", s),
    }
  }
}

// Error Handling

pub type Result<T> = std::result::Result<T, WebsocketError>;

impl error::Error for WebsocketError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match *self {
      WebsocketError::Tungstenite(ref e) => Some(e),
      WebsocketError::JSON(ref e) => Some(e),
      WebsocketError::IO(ref e) => Some(e),
      WebsocketError::Arena(ref e) => Some(e),
      WebsocketError::Other(_) => None,
    }
  }
}

impl From<tungstenite::Error> for WebsocketError {
  fn from(e: tungstenite::Error) -> Self {
    WebsocketError::Tungstenite(e)
  }
}

impl From<serde_json::Error> for WebsocketError {
  fn from(e: serde_json::Error) -> Self {
    WebsocketError::JSON(e)
  }
}

impl From<std::io::Error> for WebsocketError {
  fn from(e: std::io::Error) -> Self {
    WebsocketError::IO(e)
  }
}

impl From<ArenaError> for WebsocketError {
  fn from(e: ArenaError) -> Self {
    WebsocketError::Arena(e)
  }
}
