use std::time::Duration;

use arena::ArenaWebsocket;
use clap::{Command, Arg};
use debug::DebugWebsocket;
use jms_base::{mq::MessageQueue, kv::KVConnection};
use user::UserWebsocket;
use ws::{Websockets, WebsocketContext};

pub mod arena;
pub mod debug;
// pub mod event;
pub mod handler;
// pub mod matches;
pub mod ws;
pub mod user;

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

  
  // let file = gen_schema.get_one::<String>("file").expect("required");
  // let file = Path::new(file);
  // let schema = schemars::schema_for!(AllWebsocketMessages);
  
  // std::fs::write(file, serde_json::to_string_pretty(&schema)?)?;

  let mq = MessageQueue::new("websocket-reply").await?;
  let kv = KVConnection::new()?;

  let mut ws = Websockets::new();
  ws.register(Duration::from_millis(1000), "debug", DebugWebsocket::new()).await;
  ws.register(Duration::from_millis(1000), "arena", ArenaWebsocket::new()).await;
  ws.register(Duration::from_millis(1000), "user", UserWebsocket::new()).await;
  // ws.register(Duration::from_millis(1000), WSEventHandler {}).await;
  // ws.register(Duration::from_millis(1000), WSMatchHandler {}).await;
  // ws.register(Duration::from_millis(1000), WSArenaHandler {}).await;

  match matches.subcommand() {
    Some(("gen-schema", gen_schema)) => {
      let file = gen_schema.get_one::<String>("file").expect("required");
      // let schema = ws.to_schema();

      // std::fs::write(file, serde_json::to_string_pretty(&schema)?)?;
    },
    _ => {
      ws.begin(WebsocketContext::new(mq.channel().await?, kv)).await?;
    }
  }

  Ok(())
}