use crate::{db::{self, TableType}, models::TeamRanking};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayoffAlliance {
  pub id: usize,
  pub teams: Vec<Option<usize>>,
  pub ready: bool,
}

impl db::TableType for PlayoffAlliance {
  const TABLE: &'static str = "alliances";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    Some(self.id.into())
  }

  fn set_id(&mut self, id: Self::Id) {
    self.id = id.into();
  }
}

impl PlayoffAlliance {
  pub fn create_all(n: usize, store: &db::Store) -> db::Result<()> {
    Self::table(store)?.clear()?;

    let rankings = TeamRanking::sorted(store)?;
    let mut rankings_it = rankings.iter();

    let mut batch = db::Batch::new();
    for i in 1..=n {
      let team = rankings_it.next().map(|tr| tr.team);
      batch.insert(store, &mut PlayoffAlliance {
        id: i,
        teams: vec![ team.clone(), None, None, None ],
        ready: false
      })?;
    }
    Self::table(store)?.apply_batch(batch)?;

    Ok(())
  }

  pub fn promote(store: &db::Store) -> db::Result<()> {
    let alliances = Self::table(store)?.all()?;
    let chosen: Vec<usize> = alliances
      .iter()
      .flat_map(|a| a.teams.iter().filter_map(|x| *x))
      .collect();

    let rankings = TeamRanking::sorted(store)?;
    let mut rankings_it = rankings.iter().filter(|&r| !chosen.contains(&(r.team)));

    let mut shuffle = false;

    for i in 0..alliances.len() {
      let mut this_alliance = alliances[i].clone();
      if this_alliance.teams[0].is_none() {
        shuffle = true;
      }

      if shuffle {
        match alliances.get(i + 1) {
          Some(a) => this_alliance.teams[0] = a.teams[0],
          None => this_alliance.teams[0] = rankings_it.next().map(|r| r.team),
        }

        this_alliance.insert(store)?;
      }
    }

    Ok(())
  }
}
