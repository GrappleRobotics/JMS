use diesel::{QueryResult, RunQueryDsl, ExpressionMethods};

use crate::{db, schema::playoff_alliances};

use super::SQLJsonVector;

#[derive(Insertable, Queryable, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayoffAlliance {
  pub id: i32,
  pub teams: SQLJsonVector<usize>,
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
    let mut alliance_vec = vec![];

    for i in 1..=n {
      alliance_vec.push((
        id.eq(i as i32),
        teams.eq(SQLJsonVector(vec![] as Vec<usize>)),
        ready.eq(false)
      ));
    }

    diesel::insert_into(playoff_alliances).values(&alliance_vec).execute(conn)?;

    Ok(())
  }
}