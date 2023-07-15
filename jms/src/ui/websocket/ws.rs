use std::{time::Duration, sync::Arc};

use atomic_counter::AtomicCounter;
use futures::{StreamExt, SinkExt, stream::FuturesUnordered};
use tokio::{sync::{mpsc, Mutex}, net::TcpStream, time::{interval, Interval}};
use tokio_tungstenite::{accept_async, tungstenite};

use crate::arena::{resource::{TaggedResource, Resources}, Arena};

use super::{WebsocketMessage2UI, WebsocketMessage2JMS, SharedBroadcasts};

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

#[derive(Clone, Debug, serde::Serialize)]
#[serde(transparent)]
pub struct SerialisedMessage {
  #[serde(skip)]
  pub path: Vec<String>,
  pub message: serde_json::Value
}

impl TryFrom<&WebsocketMessage2UI> for SerialisedMessage {
  type Error = serde_json::Error;

  fn try_from(msg: &WebsocketMessage2UI) -> serde_json::Result<Self> {
    let path = msg.ws_path();

    Ok(Self {
      path: path.into_iter().map(|x| x.to_owned()).collect(),
      message: serde_json::to_value(msg)?
    })
  }
}

#[derive(Clone, Debug, serde::Serialize, schemars::JsonSchema)]
pub struct SendMeta {
  #[schemars(with = "WebsocketMessage2UI")]
  pub msg: SerialisedMessage,
  pub seq: usize,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub reply: Option<usize>,
  pub bcast: bool
}

#[derive(Clone, Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RecvMeta {
  pub msg: WebsocketMessage2JMS,
  pub seq: usize
}

#[derive(Clone)]
pub struct WebsocketContext {
  // pub bcast_tx: broadcast::Sender<SerialisedMessage>,
  pub bcast: SharedBroadcasts,
  pub handlers: SharedHandlers,
  pub arena: Arena
}

impl WebsocketContext {
  pub async fn broadcast<T: Into<WebsocketMessage2UI>>(&self, msg: T) {
    match SerialisedMessage::try_from(&msg.into()) {
      Ok(msg) => {
        // self.subscriptions.iter().any(|sub| {
        //   let subscription_str: Vec<&str> = sub.iter().map(|s| s as &str).collect();
        //   sub.len() <= msg.path.len() && subscription_str == msg.path[0..sub.len()]
        // })

        let bcast = self.bcast.lock().await;
        for (k, v) in bcast.iter() {
          let sub = k.1.iter().map(|x| x as &str).collect::<Vec<&str>>();
          if sub.len() <= msg.path.len() && sub == msg.path[0..sub.len()] {
            match v.send(msg.clone()).await {
              Ok(_) => (),
              Err(err) => error!("Could not broadcast: {}", err)
            }
          }
        }

        // if self.bcast_tx.receiver_count() > 0 {
        //   match self.bcast_tx.send(msg) {
        //     Ok(_) => (),
        //     Err(err) => error!("Could not broadcast: {}", err)
        //   }
        // }
      },
      Err(err) => error!("Could not serialise: {}", err),
      
    }
  }
}

pub struct Websocket {
  pub resource_id: Option<String>,
  pub context: WebsocketContext,
  // subscriptions: HashSet<Vec<String>>,
  send_tx: mpsc::Sender<SendMeta>,
  send_rx: mpsc::Receiver<SendMeta>,
  seq_num: atomic_counter::ConsistentCounter,
  last_recv_seq: usize
}

impl Websocket {
  pub fn new(context: WebsocketContext) -> Self {
    let (send_tx, send_rx) = mpsc::channel(128);

    Self {
      resource_id: None,
      context,
      // subscriptions: HashSet::new(),
      send_tx,
      send_rx,
      seq_num: atomic_counter::ConsistentCounter::new(0),
      last_recv_seq: 0
    }
  }

  pub async fn resource(&self) -> Option<TaggedResource> {
    // self.context.resources.lock().await.get(self.resource_id.as_deref()).cloned()
    self.context.arena.arena_impl().resources.write().await.get(self.resource_id.as_deref()).cloned()
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

  pub async fn require_fta(&self) -> anyhow::Result<()> {
    match self.is_fta().await {
      true => Ok(()),
      false => anyhow::bail!("You need to be an FTA to do that!")
    }
  }

  pub async fn send<T: Into<WebsocketMessage2UI>>(&self, msg: T) {
    match SerialisedMessage::try_from(&msg.into()) {
      Ok(msg) => match self.send_tx.send(SendMeta { msg, seq: self.seq_num.inc(), reply: None, bcast: false }).await {
        Ok(_) => (),
        Err(err) => error!("Could not send: {}", err)
      },
      Err(err) => error!("Could not serialise: {}", err),
    }
  }

  pub async fn reply<T: Into<WebsocketMessage2UI>>(&self, msg: T) {
    match SerialisedMessage::try_from(&msg.into()) {
      Ok(msg) => match self.send_tx.send(SendMeta { msg, seq: self.seq_num.inc(), reply: Some(self.last_recv_seq), bcast: false }).await {
        Ok(_) => (),
        Err(err) => error!("Could not send: {}", err)
      },
      Err(err) => error!("Could not serialise: {}", err),
    }
  }

  // pub fn is_subscribed(&self, msg: &SerialisedMessage) -> bool {
  //   if msg.path[0] == "Ping" {
  //     return true;
  //   }
    
  //   self.subscriptions.iter().any(|sub| {
  //     let subscription_str: Vec<&str> = sub.iter().map(|s| s as &str).collect();
  //     sub.len() <= msg.path.len() && subscription_str == msg.path[0..sub.len()]
  //   })
  // }

  pub async fn run(&mut self, stream: TcpStream, timeout: Duration) -> anyhow::Result<()> {
    let mut ws = accept_async(stream).await?;
    // let mut bcast_rx = self.context.bcast_tx.subscribe();
    let (bcast_tx, mut bcast_rx) = mpsc::channel(128);

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
          Some(msg) => {
            let full = serde_json::to_string(&SendMeta {
              msg, seq: self.seq_num.inc(), reply: None, bcast: true
            })?;
            ws.send(tungstenite::Message::Text(full)).await?;
          },
          None => error!("Bcast Send Closed")
          // Ok(msg) => if self.is_subscribed(&msg) {
          //   let full = serde_json::to_string(&SendMeta {
          //     msg, seq: self.seq_num.inc(), reply: None, bcast: true
          //   })?;
          //   ws.send(tungstenite::Message::Text(full)).await?;
          // },
          // Err(e) => error!("WS Broadcast Recv Error: {}", e),
        },
        recvd = self.send_rx.recv() => match recvd {
          // Unicast Message
          // Some(msg) => if msg.reply.is_some() || self.is_subscribed(&msg.msg) {
          Some(msg) => {
            ws.send(tungstenite::Message::Text(serde_json::to_string(&msg)?)).await?;
          },
          None => error!("WS Send Closed"),
        },
        recvd = ws.next() => match recvd {
          // Received from Client
          Some(recvd) => match recvd {
            Ok(msg) => match msg {
              tungstenite::Message::Text(msg_str) => {
                timeout_int.reset();

                let m: RecvMeta = serde_json::from_str(&msg_str)?;
                self.last_recv_seq = m.seq;

                let handlers = handlers_mtx.lock().await;

                match m.msg {
                  WebsocketMessage2JMS::Ping => { self.send(WebsocketMessage2UI::Pong).await },
                  WebsocketMessage2JMS::Subscribe(topic) => {
                    let mut update = false;

                    if let Some(resource_id) = self.resource_id.as_ref() {
                      let mapkey = ( resource_id.clone(), topic );
                      let mut bcasts = self.context.bcast.lock().await;
                      if !bcasts.contains_key(&mapkey) {
                        bcasts.insert(mapkey, bcast_tx.clone());
                        update = true;
                      }
                    }

                    if update {
                      // Subscriptions have updated - trigger a broadcast update for all handlers
                      for h in handlers.iter() {
                        if let Err(e) = h.handler.broadcast(&self.context).await {
                          error!("Handler broadcast error - subscription update: {}", e);
                        }
                      }
                    }
                    // if !bcasts.has_key()
                    // if !self.subscriptions.insert(topic) {
                    // }
                  },
                  _ => {
                    // Pass to handlers
                    for h in handlers.iter() {
                      if let Err(e) = h.handler.handle(&m.msg, self).await {
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
