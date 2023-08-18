use std::time::Duration;

use alliances::AlliancesWebsocket;
use arena::ArenaWebsocket;
use audience::AudienceWebsocket;
use awards::AwardsWebsocket;
use backups::BackupWebsocket;
use clap::{Command, Arg};
use components::ComponentWebsocket;
use debug::DebugWebsocket;
use event::EventWebsocket;
use jms_base::{mq::MessageQueue, kv::KVConnection};
use matches::MatchesWebsocket;
use networking::NetworkingWebsocket;
use reports::ReportWebsocket;
use scoring::ScoringWebsocket;
use tba::TBAWebsocket;
use teams::TeamWebsocket;
use tickets::TicketWebsocket;
use user::UserWebsocket;
use ws::{Websockets, WebsocketContext};

pub mod alliances;
pub mod arena;
pub mod audience;
pub mod awards;
pub mod backups;
pub mod components;
pub mod debug;
pub mod event;
pub mod handler;
pub mod matches;
pub mod scoring;
pub mod teams;
pub mod ws;
pub mod user;
pub mod reports;
pub mod tba;
pub mod networking;
pub mod tickets;

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

  let mut ws = Websockets::new();
  ws.register(Duration::from_millis(1000), "debug", DebugWebsocket::new()).await;
  ws.register(Duration::from_millis(100), "arena", ArenaWebsocket::new()).await;
  ws.register(Duration::from_millis(500), "components", ComponentWebsocket::new()).await;
  ws.register(Duration::from_millis(1000), "user", UserWebsocket::new()).await;
  ws.register(Duration::from_millis(1000), "event", EventWebsocket::new()).await;
  ws.register(Duration::from_millis(1000), "team", TeamWebsocket::new()).await;
  ws.register(Duration::from_millis(1000), "matches", MatchesWebsocket::new()).await;
  ws.register(Duration::from_millis(3000), "awards", AwardsWebsocket::new()).await;
  ws.register(Duration::from_millis(500), "scoring", ScoringWebsocket::new()).await;
  ws.register(Duration::from_millis(1000), "alliances", AlliancesWebsocket::new()).await;
  ws.register(Duration::from_millis(250), "audience", AudienceWebsocket::new()).await;
  ws.register(Duration::from_millis(100000), "reports", ReportWebsocket::new()).await;
  ws.register(Duration::from_millis(100000), "tba", TBAWebsocket::new()).await;
  ws.register(Duration::from_millis(100000), "networking", NetworkingWebsocket::new()).await;
  ws.register(Duration::from_millis(100000), "backup", BackupWebsocket::new()).await;
  ws.register(Duration::from_millis(5000), "tickets", TicketWebsocket::new()).await;

  match matches.subcommand() {
    Some(("gen-schema", gen_schema)) => {
      let file = gen_schema.get_one::<String>("file").expect("required");
      let schema = ws.to_schema();

      std::fs::write(file, serde_json::to_string_pretty(&schema)?)?;
    },
    _ => {
      // Need to set hostname since we may be in a multi-node deployment
      let mq = MessageQueue::new(&format!("websocket-reply-{}", gethostname::gethostname().to_str().unwrap())).await?;
      let kv = KVConnection::new()?;
      
      ws.begin(WebsocketContext::new(mq.channel().await?, kv)).await?;
    }
  }

  Ok(())
}