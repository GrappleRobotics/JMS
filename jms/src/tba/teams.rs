use crate::{db::{self, TableType}, models};

use super::TBAClient;

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct TBATeam(pub String);

#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct TBATeams(pub Vec<TBATeam>);

impl From<models::Team> for TBATeam {
  fn from(t: models::Team) -> Self {
    TBATeam(format!("frc{}", t.id))
  }
}

impl From<usize> for TBATeam {
  fn from(tn: usize) -> Self {
    TBATeam(format!("frc{}", tn))
  }
}

impl TBATeams {
  pub async fn issue(db: &db::Store, client: &TBAClient) -> anyhow::Result<()> {
    let teams = models::Team::all(db)?;
    let tba_teams = TBATeams(teams.iter().map(|t| TBATeam::from(t.clone())).collect());
    client.post("team_list", "update", &tba_teams).await?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn test_tba_team() {
    assert_eq!(TBATeam("frc5333".to_owned()), 5333usize.into());
    assert_eq!(TBATeam("frc4788".to_owned()), models::Team {
      id: 4788,
      name: None, affiliation: None, location: None,
      notes: None, wpakey: None, schedule: true
    }.into());
  }
}