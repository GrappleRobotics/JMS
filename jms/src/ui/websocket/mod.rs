mod arena;
mod event;
mod matches;
mod debug;

use jms_macros::define_websocket_msg;

// pub use arena::ArenaWebsocketHandler;
// pub use event::EventWebsocketHandler;
// pub use matches::MatchWebsocketHandler;
// pub use debug::DebugWebsocketHandler;

use anyhow::Result;

use futures::{SinkExt, StreamExt};
use log::{debug, error, info};
use std::{collections::HashSet, time::Duration};
use tokio::{
  net::{TcpListener, TcpStream},
  sync::broadcast,
};
use tokio_tungstenite::{accept_async, tungstenite};

use crate::{ui::websocket::{event::ws_recv_event, debug::ws_recv_debug, arena::ws_recv_arena, matches::ws_recv_match}, arena::SharedArena, schedule::worker::SharedMatchGenerators};

use self::{event::{EventMessage2UI, EventMessage2JMS}, debug::DebugMessage2JMS, arena::{ArenaMessage2UI, ArenaMessage2JMS}, matches::{MatchMessage2UI, MatchMessage2JMS}};

define_websocket_msg!($WebsocketMessage {
  send Error(String),
  recv Subscribe(Vec<String>),

  // send Debug(DebugMessage2UI),
  recv Debug(DebugMessage2JMS),

  send Event(EventMessage2UI),
  recv Event(EventMessage2JMS),

  send Arena(ArenaMessage2UI),
  recv Arena(ArenaMessage2JMS),

  send Match(MatchMessage2UI),
  recv Match(MatchMessage2JMS),
});

impl From<EventMessage2UI> for WebsocketMessage2UI {
  fn from(msg: EventMessage2UI) -> Self {
    WebsocketMessage2UI::Event(msg)
  }
}

impl From<ArenaMessage2UI> for WebsocketMessage2UI {
  fn from(msg: ArenaMessage2UI) -> Self {
    WebsocketMessage2UI::Arena(msg)
  }
}

impl From<MatchMessage2UI> for WebsocketMessage2UI {
  fn from(msg: MatchMessage2UI) -> Self {
    WebsocketMessage2UI::Match(msg)
  }
}

#[derive(Clone)]
pub struct WebsocketParams {
  pub arena: SharedArena,
  pub matches: SharedMatchGenerators
}

pub struct Websockets {
  loop_duration: Duration,
  broadcast: broadcast::Sender<Vec<WebsocketMessage2UI>>,
  params: WebsocketParams
}

impl Websockets {
  pub fn new(params: WebsocketParams, loop_duration: Duration) -> Self {
    let (tx, _) = broadcast::channel(16);

    Websockets {
      loop_duration,
      broadcast: tx,
      params
    }
  }

  pub async fn begin(&self) -> Result<()> {
    let mut update_interval = tokio::time::interval(self.loop_duration);
    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    info!("WebSocket started...");

    loop {
      tokio::select! {
        conn_result = listener.accept() => match conn_result {
          Ok((stream, _addr)) => {
            let tx = self.broadcast.clone();
            let rx = self.broadcast.subscribe();
            let params = self.params.clone();

            tokio::spawn(async move {
              if let Err(e) = connection_handler(stream, tx, rx, params).await {
                match e.downcast_ref::<tungstenite::Error>() {
                  Some(tungstenite::Error::ConnectionClosed | tungstenite::Error::Protocol(_) | tungstenite::Error::Utf8) => (),
                  _ => error!("Websocket Error: {}", e),
                }
              }
            });
          },
          Err(e) => Err(e)?,
        },

        _ = update_interval.tick() => {
          do_broadcast_update(&self.broadcast, event::ws_periodic_event().await).await?;
          do_broadcast_update(&self.broadcast, arena::ws_periodic_arena(self.params.arena.clone()).await).await?;
          do_broadcast_update(&self.broadcast, matches::ws_periodic_match(self.params.matches.clone()).await).await?;
        }
      }
    }
  }
}

async fn do_broadcast_update<T>(
  broadcast: &broadcast::Sender<Vec<WebsocketMessage2UI>>,
  messages: Result<Vec<T>>
) -> Result<()>
where
  T : Into<WebsocketMessage2UI>
{
  match messages {
    Ok(msgs) => {
      if msgs.len() > 0 && broadcast.receiver_count() > 0 {
        match broadcast.send(msgs.into_iter().map(|x| x.into()).collect()) {
          Ok(_) => (),
          Err(e) => error!("Error in broadcast: {}", e),
        }
      }
    }
    Err(e) => error!("Error in broadcast tick: {}", e),
  }
  Ok(())
}

// Can't be a self method as tokio::spawn may outlive the object itself, unless we constrain to be 'static lifetime
async fn connection_handler(
  stream: TcpStream,
  _broadcast_tx: broadcast::Sender<Vec<WebsocketMessage2UI>>,
  mut broadcast_rx: broadcast::Receiver<Vec<WebsocketMessage2UI>>,
  params: WebsocketParams
) -> Result<()> {
  let mut ws = accept_async(stream).await?;
  let mut subscribed_to = HashSet::<Vec<String>>::new();

  debug!("Websocket Connected");

  loop {
    tokio::select! {
      recvd = ws.next() => match recvd {
        Some(recvd) => match recvd {
          Ok(msg) => match msg {
            tungstenite::Message::Text(msg_str) => {
              let m: WebsocketMessage2JMS = serde_json::from_str(&msg_str)?;
              let response = match m {
                WebsocketMessage2JMS::Subscribe(schema_names) => {
                  subscribed_to.insert(schema_names);
                  Ok(vec![])
                },
                WebsocketMessage2JMS::Event(msg) => ws_recv_event(&msg).await,
                WebsocketMessage2JMS::Debug(msg) => ws_recv_debug(&msg).await,
                WebsocketMessage2JMS::Arena(msg) => ws_recv_arena(&msg, params.arena.clone()).await,
                WebsocketMessage2JMS::Match(msg) => ws_recv_match(&msg, params.matches.clone()).await
              };
              
              match response {
                Ok(resps) => {
                  if resps.len() > 0 {
                    let response_msg = serde_json::to_string(&resps)?;
                    ws.send(tungstenite::Message::Text(response_msg)).await?;
                  }
                }
                Err(err) => {
                  error!("WS Error (decode): {}", err);

                  let err_msg = WebsocketMessage2UI::Error(err.to_string());
                  let response_msg = serde_json::to_string(&vec![err_msg])?;
                  ws.send(tungstenite::Message::Text(response_msg)).await?;
                },
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
        // New broadcast 
        Ok(msgs) => {
          let msgs_filtered: Vec<&WebsocketMessage2UI> = msgs.iter().filter(|m| {
            is_subscribed_for_message(&subscribed_to, m)
            // let ts_specific = TopicSubscription { object: m.object.clone(), noun: m.noun.clone() };
            // let ts_generic = TopicSubscription { object: m.object.clone(), noun: "*".to_owned() };
            // subscriptions.contains_key(&ts_specific) || subscriptions.contains_key(&ts_generic)
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

fn is_subscribed_for_message(subscriptions: &HashSet<Vec<String>>, msg: &WebsocketMessage2UI) -> bool {
  // TODO
  // let actual_path = msg.ws_path();
  let actual_path = match msg {
    WebsocketMessage2UI::Error(_) => todo!(),
    WebsocketMessage2UI::Event(event) => [ &["Event"], event.ws_path().as_slice() ].concat(),
    WebsocketMessage2UI::Arena(arena) => [ &["Arena"], arena.ws_path().as_slice() ].concat(),
    WebsocketMessage2UI::Match(match_msg) => [ &["Match"], match_msg.ws_path().as_slice() ].concat(),
  };

  subscriptions.into_iter().any(|sub| {
    let subscription_str: Vec<&str> = sub.iter().map(|s| s as &str).collect();
    sub.len() <= actual_path.len() && subscription_str == actual_path[0..sub.len()]
  })
}
