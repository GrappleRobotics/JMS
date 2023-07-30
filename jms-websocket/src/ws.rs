use std::{time::Duration, sync::Arc, collections::{HashSet, HashMap}};

use futures::{StreamExt, SinkExt, stream::FuturesUnordered};
use jms_core_lib::models::{UserToken, MaybeToken};
use log::{error, debug, info};
use schemars::schema::{Schema, SchemaObject, RootSchema};
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
  pub path: String,
  pub data: Option<serde_json::Value>,
  pub error: Option<String>,
  pub token: Option<UserToken>
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

  pub async fn run(&mut self, stream: TcpStream, mut global_bcast: broadcast::Receiver<(String, serde_json::Value)>, timeout: Duration) -> anyhow::Result<()> {
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
            path: "ping".to_owned(),
            data: None,
            token: None
          };

          ws.send(tungstenite::Message::Text(
            serde_json::to_string(&msg)?
          )).await?;
        },
        // _ = timeout_int.tick() => {
        //   anyhow::bail!("Timed Out!");
        // },
        bcast_msg = global_bcast.recv() => match bcast_msg {
          Ok((path, msg)) => {
            if self.subscriptions.contains(&path) {
              // We're subscribed to this topic, so send it through.
              let msg = WebsocketMessage {
                message_id: uuid::Uuid::new_v4().to_string(),
                replying_to: None,
                path: path,
                data: Some(msg),
                error: None,
                token: None
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

              if msg.path == "subscribe" {
                if let Some(path) = msg.data.and_then(|x| x.as_str().map(|y| y.to_owned())) {
                    self.subscriptions.insert(path.to_owned());
                    if let Some((handler_key, method)) = path.split_once("/") {
                      // Find the handler and call on_subscribe to get the latest values
                      if let Some((_, handler)) = self.handlers.get(handler_key) {
                        match handler.on_subscribe(method).await {
                          Ok(values) => {
                            // Send the latest values directly to the websocket client
                            for (subpath, value) in values {
                              let msg = WebsocketMessage {
                                message_id: uuid::Uuid::new_v4().to_string(),
                                replying_to: None,
                                path: format!("{}/{}", handler_key, subpath),
                                data: Some(value),
                                error: None,
                                token: None,
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
              } else if let Some((handler_key, method)) = msg.path.split_once("/") {
                if let Some((_, handler)) = self.handlers.get(handler_key) {
                  let response = match handler.process_rpc_call(&self.context, &MaybeToken(msg.token), method.to_owned(), msg.data).await {
                    Ok((subpath, response)) => {
                      WebsocketMessage {
                        message_id: uuid::Uuid::new_v4().to_string(),
                        replying_to: Some(msg.message_id),
                        path: format!("{}/{}", handler_key, subpath),
                        data: Some(response),
                        error: None,
                        token: None
                      }
                    },
                    Err(e) => {
                      error!("Websocket Error: {}", e);
                      WebsocketMessage {
                        message_id: uuid::Uuid::new_v4().to_string(),
                        replying_to: Some(msg.message_id),
                        path: msg.path,
                        data: None,
                        error: Some(e.to_string()),
                        token: None
                      }
                    }
                  };

                  // Reply to the client
                  ws.send(tungstenite::Message::Text(
                    serde_json::to_string(&response)?
                  )).await?;
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
  handlers: Handlers,
  global_bcast: broadcast::Sender<(String, serde_json::Value)>
}

impl Websockets {
  pub fn new() -> Self {
    Self {
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

  // pub fn to_schema(&self) -> RootSchema {
  //   let mut generator = schemars::gen::SchemaGenerator::default();
  //   let mut obj = SchemaObject::default();
  //   obj.metadata().title = Some("TempWebsocketRootSchema".to_owned());

  //   let mut publish_schemas = vec![];
  //   let mut rpc_request_schemas = vec![];
  //   let mut rpc_response_schemas = vec![];

  //   for (key, (_, handler)) in self.handlers.iter() {
  //     if !handler.publishers().is_empty() {
  //       let mut publish_obj = SchemaObject::default();
  //       publish_obj.object().required.insert(key.clone());
  //       publish_obj.object().properties.insert(key.clone(), handler.publish_schema(&mut generator));
  //       publish_schemas.push(Schema::Object(publish_obj));
  //     }

  //     if !handler.rpcs().is_empty() {
  //       let mut rpc_request_obj = SchemaObject::default();
  //       rpc_request_obj.object().required.insert(key.clone());
  //       rpc_request_obj.object().properties.insert(key.clone(), handler.rpc_request_schema(&mut generator));
  //       rpc_request_schemas.push(Schema::Object(rpc_request_obj));

  //       let mut rpc_response_obj = SchemaObject::default();
  //       rpc_response_obj.object().required.insert(key.clone());
  //       rpc_response_obj.object().properties.insert(key.clone(), handler.rpc_response_schema(&mut generator));
  //       rpc_response_schemas.push(Schema::Object(rpc_response_obj));
  //     }
  //   }

  //   let mut publish_schema = SchemaObject::default();
  //   publish_schema.metadata().title = Some("WebsocketPublish".to_owned());
  //   publish_schema.subschemas().one_of = Some(publish_schemas);
  //   generator.definitions_mut().insert("WebsocketPublish".to_owned(), Schema::Object(publish_schema));

  //   let mut rpc_request_schema = SchemaObject::default();
  //   rpc_request_schema.metadata().title = Some("WebsocketRpcRequest".to_owned());
  //   rpc_request_schema.subschemas().one_of = Some(rpc_request_schemas);
  //   generator.definitions_mut().insert("WebsocketRpcRequest".to_owned(), Schema::Object(rpc_request_schema));

  //   let mut rpc_response_schema = SchemaObject::default();
  //   rpc_response_schema.metadata().title = Some("WebsocketRpcResponse".to_owned());
  //   rpc_response_schema.subschemas().one_of = Some(rpc_response_schemas);
  //   generator.definitions_mut().insert("WebsocketRpcResponse".to_owned(), Schema::Object(rpc_response_schema));

  //   RootSchema {
  //     meta_schema: generator.settings().meta_schema.clone(),
  //     schema: obj,
  //     definitions: generator.definitions().clone(),
  //   }
  // }

  pub async fn begin(self, ctx: WebsocketContext) -> anyhow::Result<()> {
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
            let updates = handler.update_publishers(&ctx).await;
            match updates {
              Ok(updates) => for update in updates {
                self.global_bcast.send(( format!("{}/{}", key, update.0), update.1 )).ok();
              },
              Err(e) => error!("Handler Update Error: {}: {}", key, e)
            }
          },
          None => error!("Handler broadcast wait - no fut!")
        },
        conn_result = listener.accept() => match conn_result {
          Ok((stream, _addr)) => {
            let context = ctx.clone().await?;
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
