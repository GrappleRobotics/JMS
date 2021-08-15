use diesel::{ExpressionMethods, QueryResult, RunQueryDsl};

use crate::{
  db,
  models::{SQLJson, TeamRanking},
  schema::playoff_alliances,
};

use super::SQLJsonVector;

#[derive(Insertable, Queryable, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayoffAlliance {
  pub id: i32,
  pub teams: SQLJsonVector<Option<usize>>,
  pub ready: bool,
}

impl PlayoffAlliance {
  pub fn clear(conn: &db::ConnectionT) -> QueryResult<()> {
    use crate::schema::playoff_alliances::dsl::*;
    diesel::delete(playoff_alliances).execute(conn)?;
    Ok(())
  }

  pub fn create_all(n: usize, conn: &db::ConnectionT) -> QueryResult<()> {
    use crate::schema::playoff_alliances::dsl::*;
    Self::clear(conn)?;

    let rankings = TeamRanking::get_sorted(conn)?;
    let mut rankings_it = rankings.iter();
    let mut alliance_vec = vec![];

    for i in 1..=n {
      let team = rankings_it.next().map(|tr| tr.team as usize);
      alliance_vec.push((
        id.eq(i as i32),
        teams.eq(SQLJson(vec![team, None, None, None] as Vec<Option<usize>>)),
        ready.eq(false),
      ));
    }

    diesel::insert_into(playoff_alliances)
      .values(&alliance_vec)
      .execute(conn)?;

    Ok(())
  }

  pub fn promote(conn: &db::ConnectionT) -> QueryResult<()> {
    use crate::schema::playoff_alliances::dsl::*;
    let alliances = playoff_alliances.load::<PlayoffAlliance>(conn)?;
    let chosen: Vec<usize> = alliances
      .iter()
      .flat_map(|a| a.teams.0.iter().filter_map(|x| *x))
      .collect();

    let rankings = TeamRanking::get_sorted(conn)?;
    let mut rankings_it = rankings.iter().filter(|&r| !chosen.contains(&(r.team as usize)));

    let mut shuffle = false;

    for i in 0..alliances.len() {
      let mut this_alliance = alliances[i].clone();
      if this_alliance.teams.0[0] == None {
        shuffle = true;
      }

      if shuffle {
        match alliances.get(i + 1) {
          Some(a) => this_alliance.teams.0[0] = a.teams.0[0],
          None => this_alliance.teams.0[0] = rankings_it.next().map(|r| r.team as usize),
        }

        diesel::replace_into(playoff_alliances)
          .values(&this_alliance)
          .execute(conn)?;
      }
    }

    // let mut shuffle = 0;

    // for i in 0..alliances.len() {
    //   let mut this_alliance = alliances[i].clone();
    //   if this_alliance.teams.0[0] == None || shuffle > 0 {
    //     let mut next_team = alliances.get(i + shuffle).and_then(|a| a.teams.0[0]);
    //     while next_team == None {
    //       shuffle += 1;
    //       match alliances.get(i + shuffle) {
    //         Some(a) => {
    //           next_team = a.teams.0[0];
    //         },
    //         None => {
    //           next_team = rankings_it.next().map(|r| r.team as usize);
    //           if next_team == None {
    //             break;
    //           }
    //         },
    //       }
    //     }

    //     this_alliance.teams.0[0] = next_team;
    //     diesel::replace_into(playoff_alliances)
    //       .values(&this_alliance)
    //       .execute(conn)?;
    //   }
    // }

    Ok(())
  }
}
