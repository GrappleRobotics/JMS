pub mod debug;
pub mod event;
pub mod matches;
pub mod ws;

use debug::{DebugMessage2JMS, DebugMessage2UI, WSDebugHandler};
use event::{EventMessage2JMS, EventMessage2UI, WSEventHandler};
use matches::{MatchMessage2JMS, MatchMessage2UI, WSMatchHandler};
use jms_base::{mq::{MessageQueueChannel, MessageQueue}, kv::KVConnection};
use jms_macros::define_websocket_msg;

use anyhow::Result;

use clap::{Arg, Command};
use futures::{StreamExt, stream::FuturesUnordered};
use log::{error, info};
use ws::{SerialisedMessage, WebsocketContext, DecoratedWebsocketHandler, WebsocketHandler, SendMeta, RecvMeta};
use std::{time::Duration, sync::Arc, collections::HashMap, path::Path};
use tokio::{
  net::TcpListener,
  sync::{Mutex, mpsc}, time::{interval, Interval},
};
use tokio_tungstenite::tungstenite;

use crate::ws::Websocket;

define_websocket_msg!($WebsocketMessage {
  Ping, Pong,

  send Error(String),
  recv Subscribe(Vec<String>),

  ext Debug(DebugMessage),
  ext Event(EventMessage),
  // ext Arena(ArenaMessage),
  ext Match(MatchMessage),
  // ext Resource(ResourceMessage),
  // ext Ticket(TicketMessage),
});

// (Resource ID - Topic) - Broadcast
pub type Broadcasts = HashMap<(String, Vec<String>), mpsc::Sender<SerialisedMessage>>;
pub type SharedBroadcasts = Arc<Mutex<Broadcasts>>;

// pub struct Subscription {

// };
// pub type Subscribers = Vec<>;

pub struct Websockets {
  context: WebsocketContext,
}

impl Websockets {
  pub async fn new(mq: MessageQueueChannel, kv: KVConnection) -> Self {
    // let (tx, _) = broadcast::channel(512);
    let bcast = Arc::new(Mutex::new(HashMap::new()));

    Websockets {
      context: WebsocketContext {
        bcast,
        handlers: Arc::new(Mutex::new(Vec::new())),
        mq, kv
      },
    }
  }

  pub async fn register<T: 'static + WebsocketHandler>(&self, loop_time: Duration, handler: T) {
    self.context.handlers.lock().await.push(DecoratedWebsocketHandler {
      handler: Box::new(handler),
      loop_time
    });
  }

  pub async fn begin(self) -> Result<()> {
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
        _ = ping_int.tick() => self.context.broadcast(WebsocketMessage2UI::Ping).await,
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
            let context = self.context.clone().await?;

            tokio::spawn(async move {
              let mut ws = Websocket::new(context.clone().await.unwrap());

              if let Err(e) = ws.run(stream, Duration::from_millis(5000)).await {
                match e.downcast_ref::<tungstenite::Error>() {
                  Some(tungstenite::Error::ConnectionClosed | tungstenite::Error::Protocol(_) | tungstenite::Error::Utf8) => (),
                  _ => error!("Websocket Error: {}", e),
                }
              }

              // Remove the resource when it disconnects, whether gracefully or not
              if let Some(id) = ws.resource_id {
                // context.arena.resources().write().await.remove(&id);
                let mut bcasts = context.bcast.lock().await;
                let keys = bcasts.keys().filter(|k| k.0 == id).cloned().collect::<Vec<(String, Vec<String>)>>();
                for k in keys {
                  bcasts.remove(&k);
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

#[derive(schemars::JsonSchema)]
struct AllWebsocketMessages {
  #[allow(dead_code)]
  jms2ui: WebsocketMessage2UI,
  #[allow(dead_code)]
  ui2jms: WebsocketMessage2JMS,
  #[allow(dead_code)]
  send_meta: SendMeta,
  #[allow(dead_code)]
  recv_meta: RecvMeta
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  jms_base::logging::configure(false);

  let matches = Command::new("JMS-WebSocket")
    .about("JMS WebSocket Server")
    .subcommand_required(false)
    .subcommand(
      Command::new("gen-schema")
        .long_flag("gen-schema")
        .arg(
          Arg::new("file")
            .short('f')
            .long("schema-file")
            .help("The location of the schema file to generate")
            .required(true)
        )
    ).get_matches();

  match matches.subcommand() {
    Some(("gen-schema", gen_schema)) => {
      let file = gen_schema.get_one::<String>("file").expect("required");
      let file = Path::new(file);
      let schema = schemars::schema_for!(AllWebsocketMessages);
      
      std::fs::write(file, serde_json::to_string_pretty(&schema)?)?;
      Ok(())
    },
    None => {
      let mq = MessageQueue::new("websocket-reply").await?;
      let kv = KVConnection::new().await?;

      let ws = Websockets::new(mq.channel().await?, kv).await;
      ws.register(Duration::from_millis(1000), WSDebugHandler {}).await;
      ws.register(Duration::from_millis(1000), WSEventHandler {}).await;
      ws.register(Duration::from_millis(1000), WSMatchHandler {}).await;

      ws.begin().await
    },
    _ => unreachable!()
  }

}