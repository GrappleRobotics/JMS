use jms_base::kv;
use jms_core_lib::models;

use crate::{teams::TBATeam, client::TBAClient};

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct TBAAlliance(Vec<TBATeam>);

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct TBAAlliances(Vec<TBAAlliance>);

impl From<models::PlayoffAlliance> for TBAAlliance {
  fn from(pa: models::PlayoffAlliance) -> Self {
    let teams = pa.teams
      .iter()
      .filter_map(|t| Some(*t))
      .map(|t| t.into())
      .collect();
    Self(teams)
  }
}

impl From<Vec<models::PlayoffAlliance>> for TBAAlliances {
  fn from(pas: Vec<models::PlayoffAlliance>) -> Self {
    let mut alliances = vec![];

    for pa in pas {
      alliances.push(pa.into());
    }

    Self(alliances)
  }
}

impl TBAAlliances {
  pub async fn issue(&self, kv: &kv::KVConnection) -> anyhow::Result<()> {
    TBAClient::post("alliance_selections", "update", self, kv).await
  }
}