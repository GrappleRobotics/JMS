use jms_base::kv;
use jms_core_lib::{models, db::Table};

use crate::client::TBAClient;

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct TBATeam(pub String);

#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct TBATeams(pub Vec<TBATeam>);

impl From<models::Team> for TBATeam {
  fn from(t: models::Team) -> Self {
    TBATeam(format!("frc{}", t.number))
  }
}

impl From<usize> for TBATeam {
  fn from(tn: usize) -> Self {
    TBATeam(format!("frc{}", tn))
  }
}

impl TBATeams {
  pub async fn issue(kv: &kv::KVConnection) -> anyhow::Result<()> {
    let teams = models::Team::all(kv)?;
    let tba_teams = TBATeams(teams.iter().map(|t| TBATeam::from(t.clone())).collect());
    TBAClient::post("team_list", "update", &tba_teams, kv).await?;
    Ok(())
  }
}