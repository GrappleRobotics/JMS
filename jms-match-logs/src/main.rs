use std::collections::{HashMap, hash_map::Entry};

use jms_arena_lib::{SerialisedLoadedMatch, ARENA_MATCH_KEY, MatchPlayState};
use jms_base::{kv, logging::JMSLogger};
use jms_core_lib::{models::JmsComponent, db::Table};
use jms_driverstation_lib::DriverStationReport;
use jms_match_logs_lib::{MatchLog, TimeseriesDsReportEntry};
use log::info;
use tokio::try_join;

async fn component_svc(kv: kv::KVConnection) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
  let mut component = JmsComponent::new("jms.match-logs", "JMS-Match-Logs", "L", 1000);

  component.insert(&kv)?;

  loop {
    interval.tick().await;
    component.tick(&kv)?;
  }
}

async fn logs_svc(kv: kv::KVConnection) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));

  let mut logs = HashMap::new();
  let mut last_state = None;

  loop {
    interval.tick().await;

    let reports = DriverStationReport::all(&kv)?;
    let current_match: Option<SerialisedLoadedMatch> = kv.json_get(ARENA_MATCH_KEY, "$").ok();
    if let Some(current_match) = current_match {
      let match_time = current_match.match_time.map(|mt| mt.0.num_milliseconds()).unwrap_or(0);

      if current_match.state == MatchPlayState::Auto || current_match.state == MatchPlayState::Pause || current_match.state == MatchPlayState::Teleop {
        let mut waiting: Vec<usize> = logs.keys().cloned().collect::<Vec<_>>();
        for report in reports {
          waiting.retain(|x| x != &(report.team as usize));
          // waiting.remove(report.team as usize);

          let entry = match logs.entry(report.team as usize) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(MatchLog { match_id: current_match.match_id.clone(), team: report.team as usize, timeseries: vec![] })
          };

          entry.timeseries.push(TimeseriesDsReportEntry {
            time: match_time as usize,
            report: Some(report)
          });
        }

        for wait in waiting {
          // These teams haven't received an update - make their report as nil
          if let Some(entry) = logs.get_mut(&wait) {
            entry.timeseries.push(TimeseriesDsReportEntry { time: match_time as usize, report: None });
          }
        }
      }

      if current_match.state == MatchPlayState::Complete && last_state != Some(MatchPlayState::Complete) {
        // Commit records
        info!("Committing Match Logs...");
        for (_, log) in logs.drain() {
          log.insert(&kv)?;
        }
        kv.bgsave()?;
        info!("Match Logs Committed!");
      }

      last_state = Some(current_match.state);
    } else {
      last_state = None;
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = JMSLogger::init().await?;
  
  let kv = kv::KVConnection::new()?;

  let component_fut = component_svc(kv.clone()?);
  let logs_fut = logs_svc(kv);
  try_join!(component_fut, logs_fut)?;

  Ok(())
}