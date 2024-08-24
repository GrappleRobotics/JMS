use jms_base::kv;
use jms_core_lib::{models::{self, MatchType}, db::Table, schedule::generators::QualsMatchGeneratorParams};
use log::info;

use super::quals_randomiser::{Annealer, ScheduleGenerator};

#[derive(Debug, Clone)]
pub struct QualsMatchGenerator;

impl QualsMatchGenerator {
  pub fn new() -> Self {
    Self {}
  }

  pub fn generate(
    &self,
    params: QualsMatchGeneratorParams,
    kv: &kv::KVConnection
  ) -> anyhow::Result<()> {
    let gen_start_t = chrono::Local::now();

    let station_balance_anneal = Annealer::new(1.0, 0.0, params.station_anneal_steps);
    let team_balance_anneal = Annealer::new(1.0, 0.0, params.team_anneal_steps);

    let teams: Vec<usize> = models::Team::all(kv)?.iter().filter(|&t| t.schedule).map(|t| t.number).collect();
    let existing_matches = models::Match::sorted(kv)?;

    // Determine timeslots for generation
    let mut slots = vec![];
    for block in models::ScheduleBlock::sorted(kv)? {
      match block.block_type {
        models::ScheduleBlockType::Qualification { cycle_time } => {
          let mut offset = block.start_time;
          while (offset + cycle_time.0) <= block.end_time {
            if let Some(m) = existing_matches.iter().find(|m| m.start_time >= offset && m.start_time <= (offset + cycle_time.0)) {
              // There's already a match in this slot - push back the offset by the cycle time
              offset = m.start_time + cycle_time.0;
            } else {
              // No matches in this slot - happy days.
              if offset >= chrono::Local::now() {
                slots.push( ( offset, offset + cycle_time.0 ) );
              }
              offset = offset + cycle_time.0;
            }
          }
        },
        _ => ()
      }
    }

    if slots.is_empty() {
      // Nothing to generate
      return Ok(())
    }
    
    // Generate
    let generator = ScheduleGenerator::new(teams.len(), slots.len(), 6);

    let generation_result = generator.generate(team_balance_anneal, station_balance_anneal);
    let team_sched = generation_result
      .schedule
      .contextualise(&teams);

    let gen_end_t = chrono::Local::now();
    info!("Match Generation Completed in {}", gen_end_t - gen_start_t);
      
    // Commit
    let match_n_offset = existing_matches.iter().filter(|x| x.match_type == MatchType::Qualification).map(|x| x.set_number).max().unwrap_or(0);
    for (i, col) in team_sched.0.column_iter().enumerate() {
      let teams = col.as_slice();
      let blue = teams[0..3].to_vec();
      let red = teams[3..6].to_vec();

      let set = match_n_offset + i + 1;

      let m = models::Match {
        id: models::Match::gen_id(models::MatchType::Qualification, 1, set, 1),
        name: models::Match::gen_name(models::MatchType::Qualification, 1, set, 1),
        start_time: slots[i].0,
        match_type: models::MatchType::Qualification,
        round: 1,
        set_number: set,
        match_number: 1,
        blue_teams: blue.iter().map(|&t| Some(t)).collect(),
        blue_alliance: None,
        red_teams: red.iter().map(|&t| Some(t)).collect(),
        red_alliance: None,
        dqs: vec![],
        played: false,
        ready: true
      };

      m.insert(kv)?;
    }

    Ok(())
  }
}
