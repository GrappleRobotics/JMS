use std::time::Duration;

use chrono::Local;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::{db::{DBDateTime, self}, arena::{station::{AllianceStation, AllianceStationId}, matches::{MatchPlayState, LoadedMatch}}};

use super::Match;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StampedAllianceStationStatus {
  pub time: DBDateTime,
  pub match_state: MatchPlayState,
  pub match_time: Duration,
  #[serde(flatten)]
  pub data: AllianceStation
}

impl StampedAllianceStationStatus {
  pub fn stamp(stn: AllianceStation, m: &LoadedMatch) -> Self {
    Self {
      time: Local::now().into(),
      match_state: m.current_state(),
      match_time: m.elapsed(),
      data: stn
    }
  }
}

pub type StationStatusRecord = Vec<StampedAllianceStationStatus>;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MatchStationStatusRecord {
  pub record: StationStatusRecord,
  pub station: AllianceStationId,
  #[serde(serialize_with = "super::serialize_match")]
  #[schemars(with = "super::SerializedMatch")]
  pub match_meta: Match
}

impl MatchStationStatusRecord {
  pub fn new(station: AllianceStationId, record: StationStatusRecord, m: Match) -> Self {
    Self {
      record,
      station,
      match_meta: m
    }
  }
}

impl db::TableType for MatchStationStatusRecord {
  const TABLE: &'static str = "match_station_records";
  type Id = String;

  fn id(&self) -> Option<Self::Id> {
    Some(format!("{}-{}", self.match_meta.id().unwrap_or("unknown".to_owned()), self.station.to_id()))
  }

  fn set_id(&mut self, _id: Self::Id) { }
}

