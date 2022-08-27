use std::{sync::Arc, time::Duration, path::Path, fs};

use clap::{App, Arg};
use dotenv::dotenv;
use futures::{TryFutureExt, future, FutureExt};
use jms::{arena::{self, SharedArena, resource::{SharedResources, Resources}}, config::JMSSettings, db::{self, backup::DBBackup, DBSingleton}, ds::connector::DSConnectionService, electronics::service::FieldElectronicsService, logging, tba, ui::{self, websocket::{Websockets, WebsocketMessage2UI, WebsocketMessage2JMS, resources::WSResourceHandler, matches::WSMatchHandler, event::WSEventHandler, debug::WSDebugHandler, arena::WSArenaHandler, ws::{SendMeta, RecvMeta}, tickets::WSTicketHandler}}, schedule::{worker::{MatchGenerators, MatchGenerationWorker, SharedMatchGenerators}, quals::QualsMatchGenerator, playoffs::PlayoffMatchGenerator}, models::{FTAKey, TeamRanking}, network::snmp::snmp::SNMPService, imaging::ImagingKeyService, discord};
use log::info;
use tokio::{sync::Mutex, try_join};

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
  dotenv().ok();

  let matches = App::new("JMS")
    .about("An Alternative Field-Management-System for FRC Offseason Events.")
    .arg(Arg::with_name("debug").long("debug").short("d").help("Enable debug logging."))
    .arg(
      Arg::with_name("new-cfg")
        .long("new-cfg")
        .help("Forcefully set a new configuration"),
    )
    .arg(
      Arg::with_name("cfg-only")
        .long("cfg-only")
        .help("Only load the configuration, don't start JMS"),
    )
    .arg(
      Arg::with_name("gen-schema")
        .long("gen-schema")
        .value_name("SCHEMA_FILE")
        .takes_value(true)
        .help("Only generate the JSON schema for websocket communication")
    )
    .arg(
      Arg::with_name("no-network")
        .long("no-network")
        .help("Disable Networking"),
    )
    .arg(
      Arg::with_name("no-backup")
        .long("no-backup")
        .help("Disable Backups (not recommended)"),
    )
    .arg(
      Arg::with_name("port")
        .long("port")
        .value_name("PORT")
        .takes_value(true)
        .help("Set the primary web server port")
    )
    .get_matches();

  logging::configure(matches.is_present("debug"));
  
  if let Some(v) = matches.value_of("gen-schema") {
    let file = Path::new(v);
    let schema = schemars::schema_for!(AllWebsocketMessages);
    
    fs::write(file, serde_json::to_string_pretty(&schema)?)?;
  } else if !matches.is_present("cfg-only") {

    let settings = JMSSettings::load_or_create_config(matches.is_present("new-cfg")).await?;

    db::database(); // Start connection

    FTAKey::get(&db::database())?;  // Init the FTA key if it isn't ready

    let network = match matches.is_present("no-network") {
      false => settings.network.create()?,
      true => None
    };

    let resources: SharedResources = Arc::new(Mutex::new(Resources::new()));
    let arena: SharedArena = Arc::new(Mutex::new(arena::Arena::new(3, network, resources.clone())));
    let match_workers: SharedMatchGenerators = Arc::new(Mutex::new(MatchGenerators { 
      quals: MatchGenerationWorker::new(QualsMatchGenerator::new()), 
      playoffs: MatchGenerationWorker::new(PlayoffMatchGenerator::new()) 
    }));

    let a2 = arena.clone();
    let arena_fut = async move {
      let mut interval = tokio::time::interval(Duration::from_millis(50));
      loop {
        interval.tick().await;
        a2.lock().await.update().await;
      }
      #[allow(unreachable_code)]
      Ok(())
    };

    let rankings_fut = TeamRanking::run();

    let ds_service = DSConnectionService::new(arena.clone()).await;
    let ds_fut = ds_service.run();

    let snmp_service = SNMPService::new(arena.clone());
    let snmp_fut = snmp_service.run();

    let electronics_service = FieldElectronicsService::new(arena.clone(), resources.clone(), settings.electronics).await;
    let elec_fut = electronics_service.begin();

    let ws = Websockets::new(resources.clone()).await;
    {
      ws.register(Duration::from_millis(1000), WSResourceHandler(resources.clone())).await;
      ws.register(Duration::from_millis(1000), WSMatchHandler(match_workers.clone())).await;
      ws.register(Duration::from_millis(300), WSArenaHandler(arena.clone())).await;
      ws.register(Duration::from_millis(1000), WSEventHandler {}).await;
      ws.register(Duration::from_millis(2000), WSDebugHandler {}).await;
      ws.register(Duration::from_millis(5000), WSTicketHandler {}).await;
    }
    let ws_fut = ws.begin();

    let port = match matches.value_of("port") {
      Some(str) => str.parse::<u16>()?,
      None => 80,
    };
    let web_fut = ui::web::begin(port);

    let imaging_service = ImagingKeyService::new();
    let imaging_fut = imaging_service.run().map_err(|e| anyhow::anyhow!("Imaging Service Error: {}", e));

    let mut futs = vec![];
    if let Some(tba_client) = settings.tba {
      info!("TBA Enabled");
      let tba_worker = tba::TBAWorker::new(tba_client);
      futs.push(tba_worker.begin().boxed());
    }

    if let Some(discord_conf) = settings.discord {
      info!("Discord Enabled");
      let discord_bot = discord::DiscordBot::new(discord_conf);
      futs.push(discord_bot.run().boxed());
    }

    if !matches.is_present("no-backup") {
      info!("Backups Enabled");
      let backups = DBBackup::new(arena.clone(), settings.backup);
      futs.push(backups.run().boxed());
    }

    // testing::import_team_sched().await?;

    let all_futs = future::try_join_all(futs);
    try_join!(arena_fut, rankings_fut, ds_fut, snmp_fut, elec_fut, ws_fut, web_fut, imaging_fut, all_futs)?;
  }

  Ok(())
}

// mod testing {
//   use chrono::Duration;
// use chrono::prelude::*;
//   use chrono::Date;
// use chrono::DateTime;
// use chrono::Local;
// use jms::db::TableType;
// use jms::models;
//   use jms::db;

//   pub async fn import_team_sched() -> anyhow::Result<()> {
//     let mut matches: Vec<models::Match> = models::Match::sorted(&db::database())?.into_iter().filter(|m| !m.played).collect();
//     // let vals: Vec<Vec<usize>> = vec![
//     //   vec![9058, 9024, 9516, 8890, 7113, 8876],
//     //   vec![9066, 9788, 8613, 9230, 6524, 9076],
//     //   vec![9401, 9056, 4788, 9153, 8846, 9025],
//     //   vec![9231, 9066, 9401, 8876, 9230, 7113],
//     //   vec![8890, 9231, 9025, 9024, 9153, 9788],
//     //   vec![4788, 8613, 9076, 8846, 9516, 9056],
//     //   vec![9788, 6524, 9058, 4788, 9066, 9025],
//     //   vec![8613, 9230, 7113, 9056, 9058, 9231],
//     //   vec![6524, 9401, 9153, 9076, 8890, 9516],
//     //   vec![8876, 8846, 9401, 9516, 9024, 9058],
//     //   vec![8846, 9025, 9056, 6524, 9788, 9230],
//     //   vec![9024, 9076, 8890, 9231, 4788, 9066],
//     //   vec![9153, 8613, 8876, 7113, 9056, 8890],
//     //   vec![9076, 9058, 9024, 9025, 8876, 9153],
//     //   vec![9231, 9516, 6524, 9066, 9401, 8613],
//     //   vec![9230, 7113, 9066, 9788, 4788, 8846],
//     //   vec![9516, 9788, 7113, 9401, 9025, 9076],
//     //   vec![9056, 8876, 9231, 8613, 9058, 8846],
//     //   vec![8890, 9153, 4788, 9230, 6524, 9024],
//     //   vec![9025, 9024, 9230, 9058, 8613, 9401],
//     //   vec![7113, 8890, 9788, 9056, 9076, 6524],
//     //   vec![8876, 9516, 8846, 9153, 9231, 4788],
//     // ];

//     // for (m, ts) in matches.iter_mut().zip(vals.into_iter()) {
//     //   m.blue_teams = ts[0..3].iter().map(|t| Some(*t)).collect();
//     //   m.red_teams = ts[3..6].iter().map(|t| Some(*t)).collect();
//     //   m.insert(&db::database())?;
//     // }

//     // let start_time = Local.ymd(2022, 08, 28).and_hms(09, 00, 00);
//     for (i, m) in matches.iter_mut().enumerate() {
//       // m.start_time = Some((start_time + Duration::minutes(10 * i as i64)).into());
//       let mut m2 = m.clone();
//       m.remove(&db::database())?;

//       m2.match_number = 8 + i;
//       m2.insert(&db::database())?;
//     }

//     Ok(())
//   }
// }