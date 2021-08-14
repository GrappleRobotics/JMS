mod arena;
mod event;
mod matches;

pub use arena::ArenaWebsocketHandler;
pub use event::EventWebsocketHandler;
pub use matches::MatchWebsocketHandler;

use anyhow::Result;

use futures::{lock::Mutex, SinkExt, StreamExt};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite};

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

  pub fn update(object: &str, noun: &str) -> JsonMessage {
    JsonMessage::new(object, noun, "__update__")
  }

  pub fn response(&self) -> JsonMessage {
    let mut n = self.clone();
    n.data = None;
    n.error = None;
    n
  }

  pub fn noun(&self, noun: &str) -> JsonMessage {
    let mut n = self.clone();
    n.noun = noun.to_owned();
    n
  }

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

  pub fn to_data<T>(&self, data: &T) -> Result<JsonMessage>
    where T: serde::Serialize 
  {
    Ok(self.data(serde_json::to_value(data)?))
  }

  pub fn error(&self, msg: &str) -> JsonMessage {
    let mut n = self.clone();
    n.error = Some(msg.to_owned());
    n
  }

  // pub fn unknown_noun(&self) -> anyhow::Error {
  //   anyhow!("Unknown noun")
  //   // WebsocketError::Other("Unknown noun".to_owned())
  // }

  // pub fn invalid_verb_or_data(&self) -> anyhow::Error {
  //   anyhow!("Unknown ")
  //   // WebsocketError::Other("Invalid verb/data".to_owned())
  // }
}

#[async_trait::async_trait]
pub trait WebsocketMessageHandler {
  async fn update(&mut self) -> Result<Vec<JsonMessage>>;
  async fn handle(&mut self, msg: JsonMessage) -> Result<Vec<JsonMessage>>;
}

#[derive(Hash, PartialEq, Eq)]
pub struct TopicSubscription {
  pub object: String,
  pub noun: String
}

pub struct Websockets {
  loop_duration: Duration,
  handlers: Arc<Mutex<HashMap<String, Box<dyn WebsocketMessageHandler + Send>>>>,
  broadcast: broadcast::Sender<Vec<JsonMessage>>,
}

impl Websockets {
  pub fn new(loop_duration: Duration) -> Self {
    let (tx, _) = broadcast::channel(16);

    Websockets {
      loop_duration,
      handlers: Arc::new(Mutex::new(HashMap::new())),
      broadcast: tx
    }
  }

  pub async fn register(&mut self, object_key: &'static str, handler: Box<dyn WebsocketMessageHandler + Send>) {
    let h = handler;
    self.handlers.lock().await.insert(object_key.to_owned(), h);
  }

  pub async fn begin(&self) -> Result<()> {
    let mut update_interval = tokio::time::interval(self.loop_duration);
    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    info!("WebSocket started...");

    loop {
      tokio::select! {
        conn_result = listener.accept() => match conn_result {
          Ok((stream, _addr)) => {
            let h = self.handlers.clone();
            let tx = self.broadcast.clone();
            let rx = self.broadcast.subscribe();
      
            tokio::spawn(async move {
              if let Err(e) = connection_handler(stream, tx, rx, h).await {
                match e.downcast_ref::<tungstenite::Error>() {
                    Some(tungstenite::Error::ConnectionClosed | tungstenite::Error::Protocol(_) | tungstenite::Error::Utf8) => (),
                    _ => error!("Websocket Error: {}", e),
                }
                // match e {
                //   WebsocketError::Tungstenite(ref e) => match e {
                //     tungstenite::Error::ConnectionClosed | tungstenite::Error::Protocol(_) | tungstenite::Error::Utf8 => (),
                //     err => error!("Tungstenite Error: {}", err),
                //   },
                //   err => error!("Error: {}", err),
                // }
              }
            });
          },
          Err(e) => Err(e)?,
        },

        _ = update_interval.tick() => {
          for (_, handler) in self.handlers.lock().await.iter_mut() {
            do_broadcast_update(handler, &self.broadcast).await?;
          }
        }
      }
    }
  }
}

async fn do_broadcast_update(
  handler: &mut Box<dyn WebsocketMessageHandler + Send>,
  broadcast: &broadcast::Sender<Vec<JsonMessage>>
) -> Result<()> {
  match handler.update().await {
    Ok(msgs) => {
      if msgs.len() > 0 && broadcast.receiver_count() > 0 {
        match broadcast.send(msgs) {
          Ok(_) => (),
          Err(e) => error!("Error in broadcast: {}", e),
        }
      }
    },
    Err(e) => error!("Error in handler tick: {}", e)
  }
  Ok(())
}

// Can't be a self method as tokio::spawn may outlive the object itself, unless we constrain to be 'static lifetime
async fn connection_handler(
  stream: TcpStream,
  broadcast_tx: broadcast::Sender<Vec<JsonMessage>>,
  mut broadcast_rx: broadcast::Receiver<Vec<JsonMessage>>,
  handlers: Arc<Mutex<HashMap<String, Box<dyn WebsocketMessageHandler + Send>>>>,
) -> Result<()> {
  let mut ws = accept_async(stream).await?;
  let mut subscriptions = HashMap::<TopicSubscription, ()>::new();

  debug!("Websocket Connected");

  loop {
    tokio::select! {
      recvd = ws.next() => match recvd {
        Some(recvd) => match recvd {
          Ok(msg) => match msg {
            tungstenite::Message::Text(msg_str) => {
              let m: JsonMessage = serde_json::from_str(&msg_str)?;
              if m.verb == "__subscribe__" {
                subscriptions.insert(TopicSubscription { object: m.object, noun: m.noun }, ());
              } else {
                process_incoming(&mut ws, m, &handlers, &broadcast_tx).await?;
              }
            },
            _ => ()
          },
          Err(e) => Err(e)?,
        },
        None => {
          debug!("Websocket Disconnected");
          return Ok(());
        }
      },
      recvd = broadcast_rx.recv() => match recvd {
        Ok(msgs) => {
          let msgs_filtered: Vec<&JsonMessage> = msgs.iter().filter(|m| {
            let ts_specific = TopicSubscription { object: m.object.clone(), noun: m.noun.clone() };
            let ts_generic = TopicSubscription { object: m.object.clone(), noun: "*".to_owned() };
            subscriptions.contains_key(&ts_specific) || subscriptions.contains_key(&ts_generic)
          }).collect();

          if msgs_filtered.len() > 0 {
            ws.send(tungstenite::Message::Text(serde_json::to_string(&msgs_filtered)?)).await?;
          }
        },
        Err(e) => error!("WS Broadcast Recv Error: {}", e),
      }
    }
  }
}

async fn process_incoming(
  ws: &mut WebSocketStream<TcpStream>,
  msg: JsonMessage,
  handlers: &Arc<Mutex<HashMap<String, Box<dyn WebsocketMessageHandler + Send>>>>,
  broadcast: &broadcast::Sender<Vec<JsonMessage>>
) -> Result<()> {
  let mut hs = handlers.lock().await;

  match hs.get_mut(&msg.object) {
    Some(h) => {
      let response = h.handle(msg.clone()).await;
      match response {
        Ok(msgs) => if msgs.len() > 0 {
          let response_msg = serde_json::to_string(&msgs)?;
          ws.send(tungstenite::Message::Text(response_msg)).await?;
        }
        Err(e) => {
          error!("WS Error: {}", e);
          
          let err = msg.response().error(&e.to_string());
          let response_msg = serde_json::to_string(&vec![err])?;
          ws.send(tungstenite::Message::Text(response_msg)).await?;
        }
      }
      // Send out another broadcast for this object, as WS messages coming
      // from the browser usually mutate the object
      do_broadcast_update(h, broadcast).await?;
    }
    None => {
      warn!("No WS handler for object {}", msg.object);
      let err = JsonMessage::new(&msg.object, &msg.noun, &msg.verb).error("Unknown Object");
      let response = serde_json::to_string(&vec![err])?;
      ws.send(tungstenite::Message::Text(response)).await?;
    }
  }

  Ok(())
}

// #[derive(Debug)]
// pub enum WebsocketError {
//   Tungstenite(tungstenite::Error),
//   JSON(serde_json::Error),
//   IO(std::io::Error),
//   Arena(ArenaError),
//   Other(String),
//   DBError(diesel::result::Error)
// }

// impl std::fmt::Display for WebsocketError {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     match *self {
//       WebsocketError::Tungstenite(ref e) => write!(f, "Tungstenite Error: {}", e),
//       WebsocketError::JSON(ref e) => write!(f, "JSON Error: {}", e),
//       WebsocketError::IO(ref e) => write!(f, "IO Error: {}", e),
//       WebsocketError::Arena(ref e) => write!(f, "Arena Error: {}", e),
//       WebsocketError::Other(ref s) => write!(f, "Error: {}", s),
//       WebsocketError::DBError(ref e) => write!(f, "DB Error: {}", e),
//     }
//   }
// }

// // Error Handling

// pub type Result<T> = std::result::Result<T, WebsocketError>;

// impl error::Error for WebsocketError {
//   fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//     match *self {
//       WebsocketError::Tungstenite(ref e) => Some(e),
//       WebsocketError::JSON(ref e) => Some(e),
//       WebsocketError::IO(ref e) => Some(e),
//       WebsocketError::Arena(ref e) => Some(e),
//       WebsocketError::Other(_) => None,
//       WebsocketError::DBError(ref e) => Some(e),
//     }
//   }
// }

// impl From<tungstenite::Error> for WebsocketError {
//   fn from(e: tungstenite::Error) -> Self {
//     WebsocketError::Tungstenite(e)
//   }
// }

// impl From<serde_json::Error> for WebsocketError {
//   fn from(e: serde_json::Error) -> Self {
//     WebsocketError::JSON(e)
//   }
// }

// impl From<std::io::Error> for WebsocketError {
//   fn from(e: std::io::Error) -> Self {
//     WebsocketError::IO(e)
//   }
// }

// impl From<ArenaError> for WebsocketError {
//   fn from(e: ArenaError) -> Self {
//     WebsocketError::Arena(e)
//   }
// }

// impl From<diesel::result::Error> for WebsocketError {
//   fn from(e: diesel::result::Error) -> Self {
//     WebsocketError::DBError(e)
//   }
// }
