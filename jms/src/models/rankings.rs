use diesel::{RunQueryDsl, QueryDsl, QueryResult, ExpressionMethods};

use crate::db;
use crate::schema::team_rankings;
use crate::models::Team;
use crate::scoring::scores::DerivedScore;

#[derive(Identifiable, Insertable, Queryable, Associations, AsChangeset, Debug, Clone, serde::Serialize)]
#[belongs_to(Team, foreign_key="team")]
#[primary_key(team)]
pub struct TeamRanking {
  pub team: i32,

  pub rp: i32,
  pub auto_points: i32,
  pub endgame_points: i32,
  pub teleop_points: i32,

  pub win: i32,
  pub loss: i32,
  pub tie: i32,
  pub played: i32
}

impl TeamRanking {
  pub fn get(team_num: i32, conn: &db::ConnectionT) -> QueryResult<TeamRanking> {
    use crate::schema::team_rankings::dsl::*;

    let record = team_rankings.find(team_num).first::<TeamRanking>(conn);
    match record {
      Ok(rank) => Ok(rank),
      Err(diesel::NotFound) => {
        // Insert default
        diesel::insert_into(team_rankings).values(team.eq(team_num)).execute(conn)?;

        team_rankings.find(team_num).first::<TeamRanking>(conn)
      },
      Err(e) => Err(e)
    }
  }

  pub fn update(&mut self, us_score: &DerivedScore, them_score: &DerivedScore, conn: &db::ConnectionT) -> QueryResult<()> {
    if us_score.total_score.total() > them_score.total_score.total() {
      self.rp += 2;
      self.win += 1;
    } else if us_score.total_score.total() < them_score.total_score.total() {
      self.loss += 1;
    } else {
      self.rp += 1;
      self.tie += 1;
    }

    self.rp += us_score.total_bonus_rp as i32;

    self.auto_points += us_score.total_score.auto as i32;
    self.teleop_points += us_score.total_score.teleop as i32;
    self.endgame_points += us_score.endgame_points as i32;

    self.commit(conn)
  }

  pub fn commit(&self, conn: &db::ConnectionT) -> QueryResult<()> {
    use crate::schema::team_rankings::dsl::*;

    diesel::replace_into(team_rankings).values(self).execute(conn)?;
    Ok(())
  }
}