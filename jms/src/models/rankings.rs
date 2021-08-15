use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};
use rand::Rng;

use crate::db;
use crate::models::Team;
use crate::schema::team_rankings;
use crate::scoring::scores::DerivedScore;

#[derive(
  Identifiable, Insertable, Queryable, Associations, AsChangeset, Debug, Clone, PartialEq, Eq, serde::Serialize,
)]
#[belongs_to(Team, foreign_key = "team")]
#[primary_key(team)]
pub struct TeamRanking {
  pub team: i32,

  pub rp: i32,
  pub auto_points: i32,
  pub endgame_points: i32,
  pub teleop_points: i32,
  pub random_num: i32,

  pub win: i32,
  pub loss: i32,
  pub tie: i32,
  pub played: i32,
}

impl TeamRanking {
  pub fn get(team_num: i32, conn: &db::ConnectionT) -> QueryResult<TeamRanking> {
    use crate::schema::team_rankings::dsl::*;

    let record = team_rankings.find(team_num).first::<TeamRanking>(conn);
    match record {
      Ok(rank) => Ok(rank),
      Err(diesel::NotFound) => {
        let mut rng = rand::thread_rng();

        // Insert default
        diesel::insert_into(team_rankings)
          .values((team.eq(team_num), random_num.eq::<i32>(rng.gen())))
          .execute(conn)?;

        team_rankings.find(team_num).first::<TeamRanking>(conn)
      }
      Err(e) => Err(e),
    }
  }

  pub fn update(
    &mut self,
    us_score: &DerivedScore,
    them_score: &DerivedScore,
    conn: &db::ConnectionT,
  ) -> QueryResult<()> {
    if us_score.total_score > them_score.total_score {
      self.rp += 2;
      self.win += 1;
    } else if us_score.total_score < them_score.total_score {
      self.loss += 1;
    } else {
      self.rp += 1;
      self.tie += 1;
    }

    self.rp += us_score.total_bonus_rp as i32;

    self.auto_points += us_score.mode_score.auto as i32;
    self.teleop_points += us_score.mode_score.teleop as i32;
    self.endgame_points += us_score.endgame_points as i32;

    self.played += 1;

    self.commit(conn)
  }

  pub fn commit(&self, conn: &db::ConnectionT) -> QueryResult<()> {
    use crate::schema::team_rankings::dsl::*;

    diesel::replace_into(team_rankings).values(self).execute(conn)?;
    Ok(())
  }

  pub fn get_sorted(conn: &db::ConnectionT) -> QueryResult<Vec<TeamRanking>> {
    use crate::schema::team_rankings::dsl::*;
    let mut all = team_rankings.load::<TeamRanking>(conn)?;
    all.sort();
    Ok(all)
  }
}

fn cmp_f64(a: f64, b: f64) -> std::cmp::Ordering {
  if (a - b).abs() <= 1e-10 {
    std::cmp::Ordering::Equal
  } else {
    b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal)
  }
}

fn avg(x: i32, n: i32) -> f64 {
  (x as f64) / (n as f64)
}

impl PartialOrd for TeamRanking {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for TeamRanking {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    if self == other {
      return std::cmp::Ordering::Equal;
    }

    let n_self = self.played;
    let n_other = other.played;

    // Game Manual Table 11-2
    cmp_f64(avg(self.rp, n_self), avg(other.rp, n_other))
      .then(cmp_f64(avg(self.auto_points, n_self), avg(other.auto_points, n_other)))
      .then(cmp_f64(
        avg(self.endgame_points, n_self),
        avg(other.endgame_points, n_other),
      ))
      .then(cmp_f64(
        avg(self.teleop_points, n_self),
        avg(other.teleop_points, n_other),
      ))
      .then(cmp_f64(self.random_num as f64, other.random_num as f64))
      .then(cmp_f64(self.team as f64, other.team as f64))
  }
}
