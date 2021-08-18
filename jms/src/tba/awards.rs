use crate::models;

use super::{TBAClient, teams::TBATeam};

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TBAAward {
  name_str: String,
  team_key: Option<TBATeam>,
  awardee: Option<String>
}

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TBAAwards(Vec<TBAAward>);

impl From<models::Award> for Vec<TBAAward> {
  fn from(award: models::Award) -> Self {
    award.recipients.iter().map(|recip| TBAAward { 
      name_str: award.name.clone(), 
      team_key: recip.team.map(|t| TBATeam::from(t)), 
      awardee: recip.awardee.clone() 
    }).collect()
  }
}

impl From<Vec<models::Award>> for TBAAwards {
  fn from(awards: Vec<models::Award>) -> Self {
    Self(awards.iter().flat_map(|award| Vec::<TBAAward>::from(award.clone())).collect())
  }
}

#[allow(dead_code)]
impl TBAAwards {
  pub async fn issue(&self, client: &TBAClient) -> anyhow::Result<()> {
    client.post("awards", "update", self).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn from_award() {
    let name = "My Cool Award".to_owned();
    let award = models::Award {
      id: None,
      name: name.clone(),
      recipients: vec![
        models::AwardRecipient { team: Some(5333), awardee: Some("Cool Person".to_owned()) },
        models::AwardRecipient { team: Some(4788), awardee: None },
        models::AwardRecipient { team: None, awardee: Some("Cooler Person".to_owned()) },
      ]
    };

    assert_eq!(
      vec![
        TBAAward { name_str: name.clone(), team_key: Some(TBATeam("frc5333".to_owned())), awardee: Some("Cool Person".to_owned()) },
        TBAAward { name_str: name.clone(), team_key: Some(TBATeam("frc4788".to_owned())), awardee: None },
        TBAAward { name_str: name.clone(), team_key: None, awardee: Some("Cooler Person".to_owned()) },
      ],
      Vec::<TBAAward>::from(award)
    )
  }

  #[test]
  pub fn from_awards() {
    let name1 = "My Cool Award".to_owned();
    let name2 = "My Cooler Award".to_owned();

    let awards = vec![
      models::Award {
        id: None, name: name1.clone(),
        recipients: vec![
          models::AwardRecipient { team: Some(5333), awardee: Some("Cool Person".to_owned()) },
          models::AwardRecipient { team: Some(4788), awardee: None },
        ]
      },
      models::Award {
        id: None, name: name2.clone(),
        recipients: vec![
          models::AwardRecipient { team: None, awardee: Some("Cooler Person".to_owned()) },
        ]
      },
    ];

    assert_eq!(
      TBAAwards(vec![
        TBAAward { name_str: name1.clone(), team_key: Some(TBATeam("frc5333".to_owned())), awardee: Some("Cool Person".to_owned()) },
        TBAAward { name_str: name1.clone(), team_key: Some(TBATeam("frc4788".to_owned())), awardee: None },
        TBAAward { name_str: name2.clone(), team_key: None, awardee: Some("Cooler Person".to_owned()) },
      ]),
      TBAAwards::from(awards)
    )
  }
}