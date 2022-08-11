use std::{collections::HashSet, time::Duration, sync::Arc};

use futures::{StreamExt, SinkExt, stream::FuturesUnordered};
use tokio::{sync::{broadcast, mpsc, Mutex}, net::TcpStream, time::{interval, Interval}};
use tokio_tungstenite::{accept_async, tungstenite};

use crate::arena::resource::{SharedResources, TaggedResource, Resources};

use super::{WebsocketMessage2UI, WebsocketMessage2JMS};

#[async_trait::async_trait]
pub trait WebsocketHandler : Send + Sync {
  async fn broadcast(&self, _: &WebsocketContext) -> anyhow::Result<()> { Ok(()) }
  async fn unicast(&self, _: &Websocket) -> anyhow::Result<()> { Ok(()) }
  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()>;
}

pub struct DecoratedWebsocketHandler {
  pub handler: Box<dyn WebsocketHandler>,
  pub loop_time: Duration
}

pub type Handlers = Vec<DecoratedWebsocketHandler>;
pub type SharedHandlers = Arc<Mutex<Handlers>>;

#[derive(Clone)]
pub struct SerialisedMessage {
  pub path: Vec<String>,
  pub message: String
}

impl TryFrom<&WebsocketMessage2UI> for SerialisedMessage {
  type Error = serde_json::Error;

  fn try_from(msg: &WebsocketMessage2UI) -> serde_json::Result<Self> {
    let path = match msg {
      WebsocketMessage2UI::Error(_) => vec!["Error"],
      WebsocketMessage2UI::Ping => vec!["Ping", "Ping"],
      WebsocketMessage2UI::Pong => vec!["Ping", "Pong"],
      WebsocketMessage2UI::Resource(resource) => [ &["Resource"], resource.ws_path().as_slice() ].concat(),
      WebsocketMessage2UI::Event(event) => [ &["Event"], event.ws_path().as_slice() ].concat(),
      WebsocketMessage2UI::Arena(arena) => [ &["Arena"], arena.ws_path().as_slice() ].concat(),
      WebsocketMessage2UI::Match(match_msg) => [ &["Match"], match_msg.ws_path().as_slice() ].concat(),
    };

    Ok(Self {
      path: path.into_iter().map(|x| x.to_owned()).collect(),
      message: serde_json::to_string(msg)?
    })
  }
}

#[derive(Clone)]
pub struct WebsocketContext {
  pub bcast_tx: broadcast::Sender<SerialisedMessage>,
  pub handlers: SharedHandlers,
  pub resources: SharedResources
}

impl WebsocketContext {
  pub fn broadcast<T: Into<WebsocketMessage2UI>>(&self, msg: T) {
    match SerialisedMessage::try_from(&msg.into()) {
      Ok(msg) => {
        if self.bcast_tx.receiver_count() > 0 {
          match self.bcast_tx.send(msg) {
            Ok(_) => (),
            Err(err) => error!("Could not broadcast: {}", err)
          }
        }
      },
      Err(err) => error!("Could not serialise: {}", err),
      
    }
  }
}

pub struct Websocket {
  pub resource_id: Option<String>,
  pub context: WebsocketContext,
  subscriptions: HashSet<Vec<String>>,
  send_tx: mpsc::Sender<SerialisedMessage>,
  send_rx: mpsc::Receiver<SerialisedMessage>
}

impl Websocket {
  pub fn new(context: WebsocketContext) -> Self {
    let (send_tx, send_rx) = mpsc::channel(100);

    Self {
      resource_id: None,
      context,
      subscriptions: HashSet::new(),
      send_tx,
      send_rx
    }
  }

  pub async fn resource(&self) -> Option<TaggedResource> {
    self.context.resources.lock().await.get(self.resource_id.as_deref()).cloned()
  }

  pub fn resource_mut<'a>(&self, resources: &'a mut Resources) -> Option<&'a mut TaggedResource> {
    resources.get_mut(self.resource_id.as_deref())
  }

  pub async fn is_fta(&self) -> bool {
    match self.resource().await {
      Some(r) => r.r.fta,
      None => false,
    }
  }

  pub async fn send<T: Into<WebsocketMessage2UI>>(&self, msg: T) {
    match SerialisedMessage::try_from(&msg.into()) {
      Ok(msg) => match self.send_tx.send(msg).await {
        Ok(_) => (),
        Err(err) => error!("Could not send: {}", err)
      },
      Err(err) => error!("Could not serialise: {}", err),
    }
  }

  pub fn is_subscribed(&self, msg: &SerialisedMessage) -> bool {
    if msg.path[0] == "Ping" {
      return true;
    }
    
    self.subscriptions.iter().any(|sub| {
      let subscription_str: Vec<&str> = sub.iter().map(|s| s as &str).collect();
      sub.len() <= msg.path.len() && subscription_str == msg.path[0..sub.len()]
    })
  }

  pub async fn run(&mut self, stream: TcpStream, timeout: Duration) -> anyhow::Result<()> {
    let mut ws = accept_async(stream).await?;
    let mut bcast_rx = self.context.bcast_tx.subscribe();

    let mut timeout_int = interval(timeout);
    timeout_int.reset();

    // Build intervals for each handler
    let mut handler_unicast_ints: Vec<Interval> = self.context.handlers.lock().await.iter().map(|x| interval(x.loop_time)).collect();

    debug!("Websocket Connected!");

    loop {
      // Build handler futures to yield the handler index when it's ready for an update
      let mut handler_futs = FuturesUnordered::new();
      for (i, int) in handler_unicast_ints.iter_mut().enumerate() {        
        handler_futs.push(async move {
          int.tick().await;
          i
        });
      }

      let handlers_mtx = self.context.handlers.clone();

      tokio::select! {
        _ = timeout_int.tick() => { anyhow::bail!("Timed Out"); },
        handler_idx = handler_futs.next() => match handler_idx {
          // One of the handlers has a unicast update
          Some(idx) => {
            if let Err(e) = handlers_mtx.lock().await[idx].handler.unicast(&self).await {
              self.send(WebsocketMessage2UI::Error(format!("{}", e))).await;
            }
          },
          None => error!("Handler unicast wait - no fut!"),
        },
        recvd = bcast_rx.recv() => match recvd {
          // Broadcast Message
          Ok(msg) => if self.is_subscribed(&msg) {
            ws.send(tungstenite::Message::Text(msg.message)).await?;
          },
          Err(e) => error!("WS Broadcast Recv Error: {}", e),
        },
        recvd = self.send_rx.recv() => match recvd {
          // Unicast Message
          Some(msg) => if self.is_subscribed(&msg) {
            ws.send(tungstenite::Message::Text(msg.message)).await?;
          },
          None => error!("WS Send Closed"),
        },
        recvd = ws.next() => match recvd {
          // Received from Client
          Some(recvd) => match recvd {
            Ok(msg) => match msg {
              tungstenite::Message::Text(msg_str) => {
                timeout_int.reset();

                let m: WebsocketMessage2JMS = serde_json::from_str(&msg_str)?;
                let handlers = handlers_mtx.lock().await;

                match m {
                  WebsocketMessage2JMS::Ping => { self.send(WebsocketMessage2UI::Pong).await },
                  WebsocketMessage2JMS::Subscribe(topic) => {
                    if !self.subscriptions.insert(topic) {
                      // Subscriptions have updated - trigger a broadcast update for all handlers
                      for h in handlers.iter() {
                        if let Err(e) = h.handler.broadcast(&self.context).await {
                          error!("Handler broadcast error - subscription update: {}", e);
                        }
                      }
                    }
                  },
                  _ => {
                    // Pass to handlers
                    for h in handlers.iter() {
                      if let Err(e) = h.handler.handle(&m, self).await {
                        self.send(WebsocketMessage2UI::Error(format!("{}", e))).await;
                      }
                    }
                  }
                }

              },
              _ => ()
            },
            Err(e) => Err(e)?
          },
          None => break
        }
      }
    }

    debug!("Websocket Closed Gracefully");
    Ok(())
  }
}
