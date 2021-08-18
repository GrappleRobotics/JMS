use std::error::Error;

use crate::{db::{self, DBDateTime, TableType}, models::{self, MatchGenerationRecord, MatchGenerationRecordData, ScheduleBlock}};

use super::{
  randomiser::{Annealer, GenerationResult, ScheduleGenerator, TeamSchedule},
  worker::MatchGenerator,
};

#[derive(Clone)]
pub struct QualsMatchGenerator;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct QualsMatchGeneratorParams {
  pub team_anneal_steps: usize,
  pub station_anneal_steps: usize,
}

impl QualsMatchGenerator {
  pub fn new() -> Self {
    Self {}
  }

  async fn commit_generation_record(&self, result: &GenerationResult) -> Result<(), Box<dyn Error>> {
    let mut mgr = MatchGenerationRecord {
      match_type: models::MatchType::Qualification,
      data: Some(MatchGenerationRecordData::Qualification {
        team_balance: result.team_balance_score,
        station_balance: result.station_balance_score,
        cooccurrence: result
          .cooccurrence
          .column_iter()
          .map(|col| col.iter().cloned().collect::<Vec<usize>>())
          .collect(),
        station_dist: result
          .station_dist
          .column_iter()
          .map(|col| col.iter().cloned().collect::<Vec<usize>>())
          .collect(),
      })
    };

    mgr.insert(&db::database())?;

    Ok(())
  }

  async fn commit_matches(&self, schedule: &TeamSchedule) -> Result<(), Box<dyn Error>> {
    let blocks = ScheduleBlock::qual_blocks(&db::database())?;

    models::Match::table(&db::database())?.clear()?;

    let mut match_i = 0usize;

    for block in blocks {
      for i in 0..block.num_matches() {
        let col = schedule.0.column(match_i);
        let teams = col.as_slice();
        let blue = teams[0..3].to_vec();
        let red = teams[3..6].to_vec();

        let start = block.start_time.0 + (block.cycle_time.0 * (i as i32));
        let mut m = models::Match {
          start_time: Some(DBDateTime(start)),
          match_type: models::MatchType::Qualification,
          match_subtype: None,
          set_number: 1,
          match_number: 1,
          blue_teams: blue.iter().map(|&t| Some(t)).collect(),
          blue_alliance: None,
          red_teams: red.iter().map(|&t| Some(t)).collect(),
          red_alliance: None,
          score: None,
          score_time: None,
          winner: None,
          played: false,
        };
        m.insert(&db::database())?;
        match_i += 1;
      }
    }

    Ok(())
  }
}

#[async_trait::async_trait]
impl MatchGenerator for QualsMatchGenerator {
  type ParamType = QualsMatchGeneratorParams;

  fn match_type(&self) -> models::MatchType {
    models::MatchType::Qualification
  }

  async fn generate(
    &self,
    params: QualsMatchGeneratorParams,
    _: Option<MatchGenerationRecord>,
  ) -> Result<(), Box<dyn Error>> {
    let station_balance_anneal = Annealer::new(1.0, 0.0, params.station_anneal_steps);
    let team_balance_anneal = Annealer::new(1.0, 0.0, params.team_anneal_steps);

    let teams: Vec<usize> = models::Team::all(&db::database())?.iter().filter(|&t| t.schedule).map(|t| t.id).collect();

    // Generate
    let num_matches = ScheduleBlock::qual_blocks(&db::database())?
      .iter()
      .map(|x| x.num_matches())
      .sum();

    let generator = ScheduleGenerator::new(teams.len(), num_matches, 6);

    let generation_result = generator.generate(team_balance_anneal, station_balance_anneal);
    let team_sched = generation_result
      .schedule
      .contextualise(&teams);

    // Commit
    self.commit_generation_record(&generation_result).await?;
    self.commit_matches(&team_sched).await?;

    Ok(())
  }
}
