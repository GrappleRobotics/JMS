use std::error::Error;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::{db, models::{self, MatchGenerationRecord, MatchGenerationRecordData, SQLDatetime, SQLJson, ScheduleBlock}};

use super::{randomiser::{Annealer, GenerationResult, ScheduleGenerator, TeamSchedule}, worker::MatchGenerator};

#[derive(Clone)]
pub struct QualsMatchGenerator;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct QualsMatchGeneratorParams {
  pub team_anneal_steps: usize,
  pub station_anneal_steps: usize
}

impl QualsMatchGenerator {
  pub fn new() -> Self {
    Self { }
  }

  async fn commit_generation_record(&self, result: &GenerationResult) -> Result<(), Box<dyn Error>> {
    use crate::schema::match_generation_records::dsl::*;

    diesel::replace_into(match_generation_records)
      .values(MatchGenerationRecord {
        match_type: models::MatchType::Qualification,
        data: Some(SQLJson(MatchGenerationRecordData::Qualification {
          team_balance: result.team_balance_score,
          station_balance: result.station_balance_score,
          cooccurrence: SQLJson(result.cooccurrence.column_iter().map(|col| col.iter().cloned().collect::<Vec<usize>>() ).collect()),
          station_dist: SQLJson(result.station_dist.column_iter().map(|col| col.iter().cloned().collect::<Vec<usize>>() ).collect())
        }))
      })
      .execute(&db::connection())?;

    Ok(())
  }

  async fn commit_matches(&self, schedule: &TeamSchedule) -> Result<(), Box<dyn Error>> {
    use crate::schema::matches::dsl::*;
    let blocks = ScheduleBlock::qual_blocks(&db::connection())?;

    let mut match_vec = vec![];
    let mut match_i = 0usize;

    for block in blocks {
      for i in 0..block.num_matches() {
        let col = schedule.0.column(match_i);
        let teams = col.as_slice();
        let blue = teams[0..3].to_vec();
        let red = teams[3..6].to_vec();
        
        let start = block.start_time.0 + (block.cycle_time.0 * (i as i32));
        match_vec.push((
          start_time.eq(Some(SQLDatetime(start))),
          match_type.eq(models::MatchType::Qualification),
          set_number.eq(0),
          match_number.eq((match_i + 1) as i32),
          blue_teams.eq(SQLJson(blue)),
          red_teams.eq(SQLJson(red)),
          played.eq(false)
        ));
        match_i += 1;
      }
    }

    diesel::delete(matches.filter(match_type.eq(models::MatchType::Qualification))).execute(&db::connection())?;
    diesel::insert_into(matches).values(&match_vec).execute(&*db::connection())?;

    Ok(())
  }
}

#[async_trait::async_trait]
impl MatchGenerator for QualsMatchGenerator {
  type ParamType = QualsMatchGeneratorParams;

  fn match_type(&self) -> models::MatchType {
    models::MatchType::Qualification
  }

  async fn generate(&self, params: QualsMatchGeneratorParams, _: Option<MatchGenerationRecord>) -> Result<(), Box<dyn Error>> {
    let station_balance_anneal = Annealer::new(1.0, 0.0, params.station_anneal_steps);
    let team_balance_anneal = Annealer::new(1.0, 0.0, params.team_anneal_steps);

    let teams = {
      use crate::schema::teams::dsl::*;
      teams.select(id).get_results::<i32>(&db::connection())?
    };

    // Generate
    let num_matches = ScheduleBlock::qual_blocks(&db::connection())?.iter().map(|x| x.num_matches()).sum();

    let generator = ScheduleGenerator::new(teams.len(), num_matches, 6);

    let generation_result = generator.generate(team_balance_anneal, station_balance_anneal);
    let team_sched = generation_result.schedule.contextualise(&teams.iter().map(|&x| x as u16).collect::<Vec<u16>>());

    // Commit
    self.commit_generation_record(&generation_result).await?;
    self.commit_matches(&team_sched).await?;

    Ok(())
  }
}

