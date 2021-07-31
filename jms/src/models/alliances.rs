use diesel::{QueryResult, RunQueryDsl, ExpressionMethods};

use crate::{db, models::{SQLJson, TeamRanking}, schema::playoff_alliances};

use super::SQLJsonVector;

#[derive(Insertable, Queryable, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayoffAlliance {
  pub id: i32,
  pub teams: SQLJsonVector<Option<usize>>,
  pub ready: bool
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
        ready.eq(false)
      ));
    }

    diesel::insert_into(playoff_alliances).values(&alliance_vec).execute(conn)?;

    Ok(())
  }
}