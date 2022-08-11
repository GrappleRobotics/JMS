pub mod arena;
pub mod event;
pub mod matches;
pub mod debug;
pub mod resources;
pub mod ws;

use jms_macros::define_websocket_msg;

use anyhow::Result;

use futures::{StreamExt, stream::FuturesUnordered};
use log::{error, info};
use std::{time::Duration, sync::Arc};
use tokio::{
  net::TcpListener,
  sync::{broadcast, Mutex}, time::{interval, Interval},
};
use tokio_tungstenite::tungstenite;

use crate::{arena::resource::SharedResources, ui::websocket::ws::Websocket};

use self::{event::{EventMessage2UI, EventMessage2JMS}, debug::DebugMessage2JMS, arena::{ArenaMessage2UI, ArenaMessage2JMS}, matches::{MatchMessage2UI, MatchMessage2JMS}, resources::{ResourceMessage2UI, ResourceMessage2JMS}, ws::{WebsocketHandler, DecoratedWebsocketHandler, WebsocketContext}};

define_websocket_msg!($WebsocketMessage {
  Ping, Pong,

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

  send Resource(ResourceMessage2UI),
  recv Resource(ResourceMessage2JMS),
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

impl From<ResourceMessage2UI> for WebsocketMessage2UI {
  fn from(msg: ResourceMessage2UI) -> Self {
    WebsocketMessage2UI::Resource(msg)
  }
}

pub struct Websockets {
  context: WebsocketContext,
}

impl Websockets {
  pub async fn new(resources: SharedResources) -> Self {
    let (tx, _) = broadcast::channel(512);

    Websockets {
      context: WebsocketContext {
        bcast_tx: tx,
        handlers: Arc::new(Mutex::new(Vec::new())),
        resources: resources.clone()
      },
    }
  }

  pub async fn register<T: 'static + WebsocketHandler>(&self, loop_time: Duration, handler: T) {
    self.context.handlers.lock().await.push(DecoratedWebsocketHandler {
      handler: Box::new(handler),
      loop_time
    });
  }

  pub async fn begin(&self) -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    info!("WebSockets started...");

    // Build intervals for each handler
    let mut handler_ints: Vec<Interval> = self.context.handlers.lock().await.iter().map(|x| interval(x.loop_time)).collect();

    let mut ping_int = interval(Duration::from_millis(250));

    loop {
      // Build handler futures to yield the handler index when it's ready for an update
      let mut handler_futs = FuturesUnordered::new();
      for (i, int) in handler_ints.iter_mut().enumerate() {        
        handler_futs.push(async move {
          int.tick().await;
          i
        });
      }

      tokio::select! {
        _ = ping_int.tick() => self.context.broadcast(WebsocketMessage2UI::Ping),
        handler_idx = handler_futs.next() => match handler_idx {
          // One of the handlers has a broadcast update
          Some(idx) => {
            if let Err(e) = self.context.handlers.lock().await[idx].handler.broadcast(&self.context).await {
              error!("WS Broadcast Error: {}", e);
            }
          },
          None => error!("Handler broadcast wait - no fut!")
        },
        conn_result = listener.accept() => match conn_result {
          Ok((stream, _addr)) => {
            let context = self.context.clone();

            tokio::spawn(async move {
              let mut ws = Websocket::new(context.clone());

              if let Err(e) = ws.run(stream, Duration::from_millis(1500)).await {
                match e.downcast_ref::<tungstenite::Error>() {
                  Some(tungstenite::Error::ConnectionClosed | tungstenite::Error::Protocol(_) | tungstenite::Error::Utf8) => (),
                  _ => error!("Websocket Error: {}", e),
                }
              }

              // Remove the resource when it disconnects, whether gracefully or not
              if let Some(id) = ws.resource_id {
                context.resources.lock().await.remove(&id);
              }
            });
          },
          Err(e) => Err(e)?,
        }
      }
    }
  }
}

