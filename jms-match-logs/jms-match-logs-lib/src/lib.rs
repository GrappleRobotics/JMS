use std::convert::Infallible;

use jms_core_lib::db::Table;
use jms_driverstation_lib::DriverStationReport;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MatchLog {
  pub team: usize,
  pub match_id: String,
  pub timeseries: Vec<TimeseriesDsReportEntry>
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TimeseriesDsReportEntry {
  pub time: usize,      // In ms
  pub report: Option<DriverStationReport>
}

impl Table for MatchLog {
  const PREFIX: &'static str = "db:match_logs";
  type Id = String;
  type Err = Infallible;

  fn id(&self) -> Self::Id {
    format!("{}:{}", self.match_id, self.team)
  }
}