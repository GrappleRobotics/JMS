use std::{time::Duration, sync::Arc, collections::{HashSet, HashMap}};

use futures::{StreamExt, SinkExt, stream::FuturesUnordered};
use log::{error, debug, info};
use serde_json::json;
use tokio::{sync::broadcast, net::{TcpStream, TcpListener}, time::{interval, Interval}};
use tokio_tungstenite::{accept_async, tungstenite};

use crate::handler::WebsocketHandler;

// type SharedHandlers = Arc<RwLock<HashMap<String, (Duration, Box<dyn WebsocketHandler + Send + Sync>)>>>;
type Handlers = HashMap<String, (Duration, Box<dyn WebsocketHandler + Send + Sync>)>;
type SharedHandlers = Arc<Handlers>;

pub struct WebsocketContext {
  pub mq: jms_base::mq::MessageQueueChannel,
  pub kv: jms_base::kv::KVConnection,
}

impl WebsocketContext {
  pub fn new(mq: jms_base::mq::MessageQueueChannel, kv: jms_base::kv::KVConnection) -> Self {
    Self { mq, kv }
  }

  pub async fn clone(&self) -> anyhow::Result<Self> {
    Ok(Self {
      mq: self.mq.clone().await?,
      kv: self.kv.clone()?
    })
  }
}

#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct WebsocketMessage {
  pub message_id: String,
  pub replying_to: Option<String>,
  pub data: serde_json::Value,
  pub error: Option<String>
}

pub struct Websocket {
  pub context: WebsocketContext,
  pub subscriptions: HashSet<String>,
  handlers: SharedHandlers
}

impl Websocket {
  pub fn new(ctx: WebsocketContext, handlers: SharedHandlers) -> Self {
    Self {
      context: ctx,
      subscriptions: HashSet::new(),
      handlers
    }
  }

  pub async fn run(&mut self, stream: TcpStream, mut global_bcast: broadcast::Receiver<serde_json::Value>, timeout: Duration) -> anyhow::Result<()> {
    let mut ws = accept_async(stream).await?;

    let mut timeout_int = interval(timeout);
    timeout_int.reset();

    let mut ping_int = interval(Duration::from_millis(250));

    debug!("Websocket Connected!");

    loop {
      tokio::select! {
        _ = ping_int.tick() => {
          let msg = WebsocketMessage {
            message_id: uuid::Uuid::new_v4().to_string(),
            replying_to: None,
            error: None,
            data: json!("ping"),
          };

          ws.send(tungstenite::Message::Text(
            serde_json::to_string(&msg)?
          )).await?;
        },
        // _ = timeout_int.tick() => {
        //   anyhow::bail!("Timed Out!");
        // },
        bcast_msg = global_bcast.recv() => match bcast_msg {
          Ok(msg) => {
            // Traverse the first two levels of the dict, since those contain the handler key, and then
            // the topic. This should never actually panic due to how things are formatted on the Websockets side.
            let (handler_key, inner) = msg.as_object().unwrap().into_iter().next().unwrap();
            let topic_key = inner.as_object().unwrap().keys().next().unwrap();

            let subscription_key = format!("{}/{}", handler_key, topic_key);

            if self.subscriptions.contains(&subscription_key) {
              // We're subscribed to this topic, so send it through.
              let msg = WebsocketMessage {
                message_id: uuid::Uuid::new_v4().to_string(),
                replying_to: None,
                data: msg,
                error: None
              };

              ws.send(tungstenite::Message::Text(
                serde_json::to_string(&msg)?
              )).await?;
            }
          },
          Err(_) => break Ok(())  /* Broadcast channel closed */
        },
        ws_msg = ws.next() => match ws_msg {
          Some(Ok(msg)) => match msg {
            tungstenite::Message::Text(msg) => {
              timeout_int.reset();
              let msg: WebsocketMessage = serde_json::from_str(&msg)?;

              if let Some(data) = msg.data.as_object() {
                // Try to find which handler responds. Data will have a single key, based on the handler.
                if let Some((key, value)) = data.into_iter().next() {
                  if key == "subscribe" {
                    // Subscribe to the requested topic
                    if let Some(value) = value.as_str() {
                      self.subscriptions.insert(value.to_owned());
                      if let Some((handler_key, method)) = value.split_once("/") {
                        // Find the handler and call on_subscribe to get the latest values
                        if let Some((_, handler)) = self.handlers.get(handler_key) {
                          match handler.on_subscribe(method).await {
                            Ok(values) => {
                              // Send the latest values directly to the websocket client
                              for value in values {
                                let msg = WebsocketMessage {
                                  message_id: uuid::Uuid::new_v4().to_string(),
                                  replying_to: None,
                                  data: json!({ handler_key: value }),
                                  error: None
                                };

                                ws.send(tungstenite::Message::Text(
                                  serde_json::to_string(&msg)?
                                )).await?;
                              }
                            },
                            Err(e) => {
                              error!("on_subscribe error: {}", e);
                            }
                          }
                        }
                      }
                    }
                  } else if let Some((_, handler)) = self.handlers.get(key) {
                    // Process the RPC Call
                    let response = match handler.process_rpc_call(&self.context, value.clone()).await {
                      Ok(response) => {
                        WebsocketMessage {
                          message_id: uuid::Uuid::new_v4().to_string(),
                          replying_to: Some(msg.message_id),
                          data: json!({ key: response }),
                          error: None
                        }
                      },
                      Err(e) => {
                        error!("Websocket Error: {}", e);
                        WebsocketMessage {
                          message_id: uuid::Uuid::new_v4().to_string(),
                          replying_to: Some(msg.message_id),
                          data: json!(null),
                          error: Some(e.to_string())
                        }
                      }
                    };

                    // Reply to the client
                    ws.send(tungstenite::Message::Text(
                      serde_json::to_string(&response)?
                    )).await?;
                  }
                }
              }
            },
            _ => ()
          },
          Some(Err(e)) => Err(e)?,
          None => break Ok(()) /* Shutdown Gracefully */
        }
      }
    }
  }
}

pub struct Websockets {
  context: WebsocketContext,
  handlers: Handlers,
  global_bcast: broadcast::Sender<serde_json::Value>
}

impl Websockets {
  pub fn new(ctx: WebsocketContext) -> Self {
    Self {
      context: ctx,
      handlers: HashMap::new(),
      global_bcast: broadcast::channel(32).0
    }
  }

  pub async fn register<T: 'static + WebsocketHandler + Send + Sync>(&mut self, loop_time: Duration, name: &str, handler: T) {
    self.handlers.insert(name.to_owned(), (
      loop_time,
      Box::new(handler)
    ));
  }

  pub async fn begin(self) -> anyhow::Result<()> {
    info!("Starting Websockets...");
    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    info!("Websockets started...");

    let handlers = Arc::new(self.handlers);

    // Build intervals for each handler
    let mut handler_ints: Vec<(String, Interval)> = handlers.iter().map(|(k, v)| (k.clone(), interval(v.0))).collect();

    loop {
      // Build handler futures to yield the handler index when it's ready for an update
      let mut handler_futs = FuturesUnordered::new();
      for (k, int) in handler_ints.iter_mut() {        
        handler_futs.push(async move {
          int.tick().await;
          k
        });
      }

      tokio::select! {
        handler_idx = handler_futs.next() => match handler_idx {
          // One of the handlers has a broadcast update
          Some(key) => {
            let (_, handler) = handlers.get(key).unwrap();
            let updates = handler.update_publishers(&self.context).await;
            match updates {
              Ok(updates) => for update in updates {
                let bcast_value = json!({ key.to_owned(): update });
                self.global_bcast.send(bcast_value).ok();
              },
              Err(e) => error!("Handler Update Error: {}: {}", key, e)
            }
          },
          None => error!("Handler broadcast wait - no fut!")
        },
        conn_result = listener.accept() => match conn_result {
          Ok((stream, _addr)) => {
            let context = self.context.clone().await?;
            let global_bcast_rx = self.global_bcast.subscribe();
            let handlers = handlers.clone();

            tokio::spawn(async move {
              let mut ws = Websocket::new(context.clone().await.unwrap(), handlers);

              if let Err(e) = ws.run(stream, global_bcast_rx, Duration::from_millis(5000)).await {
                match e.downcast_ref::<tungstenite::Error>() {
                  Some(tungstenite::Error::ConnectionClosed | tungstenite::Error::Protocol(_) | tungstenite::Error::Utf8) => (),
                  _ => error!("Websocket Error: {}", e),
                }
              }
            });
          },
          Err(e) => Err(e)?,
        }
      }
    }
  }
}
