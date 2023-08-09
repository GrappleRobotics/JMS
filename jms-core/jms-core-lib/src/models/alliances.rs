use std::num::ParseIntError;

use jms_base::kv;

use crate::db::Table;

use super::TeamRanking;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PlayoffAlliance {
  pub number: usize,
  pub teams: Vec<usize>,
}

#[async_trait::async_trait]
impl Table for PlayoffAlliance {
  const PREFIX: &'static str = "db:alliance";
  type Id = usize;
  type Err = ParseIntError;

  fn id(&self) -> Self::Id {
    return self.number
  }
}

impl PlayoffAlliance {
  pub fn sorted(kv: &kv::KVConnection) -> anyhow::Result<Vec<PlayoffAlliance>> {
    let mut all = Self::all(&kv)?;
    all.sort_by(|a, b| a.number.cmp(&b.number));
    Ok(all)
  }

  pub fn create_all(n: usize, kv: &kv::KVConnection) -> anyhow::Result<()> {
    Self::clear(kv)?;

    let rankings = TeamRanking::sorted(kv)?;
    let mut rankings_it = rankings.iter();

    // let mut batch = db::Batch::new();
    for i in 1..=n {
      let team = rankings_it.next().map(|tr| tr.team);
      if let Some(team) = team {
        PlayoffAlliance {
          number: i,
          teams: vec![ team ],
        }.insert(kv)?;
      } else {
        PlayoffAlliance {
          number: i,
          teams: vec![]
        }.insert(kv)?;
      }
    }

    Ok(())
  }

  pub fn promote(kv: &kv::KVConnection) -> anyhow::Result<()> {
    // let alliances = Self::table(store)?.all()?;
    let alliances = Self::sorted(kv)?;
    let chosen: Vec<usize> = alliances
      .iter()
      .flat_map(|a| a.teams.iter().map(|x| *x))
      .collect();

    let rankings = TeamRanking::sorted(kv)?;
    let mut rankings_it = rankings.iter().filter(|&r| !chosen.contains(&(r.team)));

    let mut shuffle = false;

    for i in 0..alliances.len() {
      let mut this_alliance = alliances[i].clone();
      if this_alliance.teams.is_empty() {
        shuffle = true;
      }

      if shuffle {
        match alliances.get(i + 1) {
          Some(a) => match this_alliance.teams.len() {
            0 => this_alliance.teams.push(a.teams[0]),
            _ => this_alliance.teams[0] = a.teams[0],
          },
          None => match rankings_it.next().map(|r| r.team) {
            Some(t) => match this_alliance.teams.len() {
              0 => this_alliance.teams.push(t),
              _ => this_alliance.teams[0] = t,
            },
            None => ()
          },
        }

        this_alliance.insert(kv)?;
      }
    }

    Ok(())
  }
}
