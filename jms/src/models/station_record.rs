use std::time::Duration;

use chrono::Local;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::{db::{DBDateTime, self, Json}, arena::{station::{AllianceStation, AllianceStationId}, matches::{MatchPlayState, LoadedMatch}}};

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
pub struct MatchStationStatusRecordKey {
  pub team: Option<u16>,
  pub station: AllianceStationId,
  pub match_id: String
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MatchStationStatusRecord {
  pub record: StationStatusRecord,
  pub key: MatchStationStatusRecordKey
}

impl MatchStationStatusRecord {
  pub fn new(station: &AllianceStation, record: StationStatusRecord, match_id: String) -> Self {
    Self {
      record,
      key: MatchStationStatusRecordKey {
        station: station.station,
        team: station.team,
        match_id
      }
    }
  }
}

impl db::TableType for MatchStationStatusRecord {
  const TABLE: &'static str = "match_station_records";
  type Id = Json<MatchStationStatusRecordKey>;

  fn id(&self) -> Option<Self::Id> {
    Some(self.key.clone().into())
  }
}

