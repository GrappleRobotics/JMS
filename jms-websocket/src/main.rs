use std::time::Duration;

use clap::{Command, Arg};
use debug::DebugWebsocket;
use jms_base::{mq::MessageQueue, kv::KVConnection};
use ws::{Websockets, WebsocketContext};

// pub mod arena;
pub mod debug;
// pub mod event;
pub mod handler;
// pub mod matches;
pub mod ws;

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
      // let file = gen_schema.get_one::<String>("file").expect("required");
      // let file = Path::new(file);
      // let schema = schemars::schema_for!(AllWebsocketMessages);
      
      // std::fs::write(file, serde_json::to_string_pretty(&schema)?)?;
      Ok(())
    },
    None => {
      let mq = MessageQueue::new("websocket-reply").await?;
      let kv = KVConnection::new()?;

      let mut ws = Websockets::new(WebsocketContext::new(mq.channel().await?, kv));
      ws.register(Duration::from_millis(1000), "debug", DebugWebsocket::new()).await;
      // ws.register(Duration::from_millis(1000), WSDebugHandler {}).await;
      // ws.register(Duration::from_millis(1000), WSEventHandler {}).await;
      // ws.register(Duration::from_millis(1000), WSMatchHandler {}).await;
      // ws.register(Duration::from_millis(1000), WSArenaHandler {}).await;

      ws.begin().await?;
      Ok(())
    },
    _ => unreachable!()
  }

}