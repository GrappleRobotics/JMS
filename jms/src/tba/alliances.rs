use crate::models;

use super::{TBAClient, teams::TBATeam};

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct TBAAlliance(Vec<TBATeam>);

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct TBAAlliances(Vec<TBAAlliance>);

impl From<models::PlayoffAlliance> for TBAAlliance {
  fn from(pa: models::PlayoffAlliance) -> Self {
    if !pa.ready {
      return Self(vec![]);
    }
    
    let teams = pa.teams
      .iter()
      .filter_map(|t| *t)
      .map(|t| t.into())
      .collect();
    Self(teams)
  }
}

impl From<Vec<models::PlayoffAlliance>> for TBAAlliances {
  fn from(pas: Vec<models::PlayoffAlliance>) -> Self {
    let mut alliances = vec![];

    if pas.iter().all(|pa| pa.ready) {
      for pa in pas {
        alliances.push(pa.into());
      }
    }

    Self(alliances)
  }
}

impl TBAAlliances {
  pub async fn issue(&self, client: &TBAClient) -> anyhow::Result<()> {
    client.post("alliance_selections", "update", self).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_from_ready_alliance() {
    let alliance = models::PlayoffAlliance {
      id: 1,
      teams: vec![ Some(100), None, Some(120) ],
      ready: true
    };
    
    assert_eq!(
      TBAAlliance(vec![
        TBATeam("frc100".to_owned()), 
        TBATeam("frc120".to_owned())
      ]),
      TBAAlliance::from(alliance)
    )
  }

  #[test]
  fn test_from_not_ready_alliance() {
    let alliance = models::PlayoffAlliance {
      id: 1,
      teams: vec![ Some(100), None, Some(120) ],
      ready: false
    };
    
    assert_eq!(
      TBAAlliance(vec![]),
      TBAAlliance::from(alliance)
    )
  }

  #[test]
  fn test_from_alliances_all_ready() {
    let alliances = vec![
      models::PlayoffAlliance { id: 1, ready: true, teams: vec![ Some(100), None, Some(120) ] },
      models::PlayoffAlliance { id: 2, ready: true, teams: vec![ Some(4788), Some(5333), Some(5663) ] },
    ];

    assert_eq!(
      TBAAlliances(vec![
        alliances[0].clone().into(),
        alliances[1].clone().into()
      ]),
      alliances.into()
    )
  }

  #[test]
  fn test_from_alliances_some_not_ready() {
    let alliances = vec![
      models::PlayoffAlliance { id: 1, ready: true, teams: vec![ Some(100), None, Some(120) ] },
      models::PlayoffAlliance { id: 2, ready: false, teams: vec![ Some(4788), Some(5333), Some(5663) ] },
    ];

    assert_eq!(
      TBAAlliances(vec![]),
      alliances.into()
    )
  }
}