use chrono::Duration;
use jms_base::kv;
use jms_core_lib::{db::{Singleton, Table}, models::{Award, AwardRecipient, CommittedMatchScores, Match, MatchType, PlayoffAlliance, PlayoffMode, PlayoffModeType, ScheduleBlock, ScheduleBlockType, Team}, scoring::scores::ScoringConfig};
use log::{info, warn};

use super::bracket::bracket_update;

#[derive(Debug)]
pub enum GenerationUpdate {
  MatchUpdates(Vec<PlayoffScheduleItem>),
  TournamentWon {
    winner: usize,
    finalist: usize,
  },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayoffAllianceDescriptor {
  Alliance(usize),
  WinnerOf(MatchType, /* round */ usize, /* set */ usize),
  LoserOf(MatchType, /* round */ usize, /* set */ usize),
  Bye
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayoffScheduleItem {
  Match(IncompleteMatch),
  TiebreakerSlot,
  AwardsBreak,
}

#[derive(Debug, Clone)]
pub struct HydratedPlayoffScheduleItem {
  pub duration: Option<chrono::Duration>,
  pub item: PlayoffScheduleItem
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncompleteMatch {
  pub ty: MatchType,
  pub round: usize,
  pub set: usize,
  pub match_num: usize,
  pub red: PlayoffAllianceDescriptor,
  pub blue: PlayoffAllianceDescriptor,
}

#[derive(Debug, Clone)]
pub struct PlayoffMatchGenerator { }

impl PlayoffMatchGenerator {
  pub fn reset(kv: &kv::KVConnection) -> anyhow::Result<()> {
    let matches = Match::all(kv)?;
    if matches.iter().any(|x| x.played && (x.match_type == MatchType::Playoff || x.match_type == MatchType::Final)) {
      anyhow::bail!("Can't reset the Playoffs Schedule if it's already in motion!");
    }

    let to_delete = matches.iter()
      .filter(|m| m.match_type == MatchType::Playoff || m.match_type == MatchType::Final);
    
    to_delete
      .map(|m| m.delete(&kv))
      .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(())
  }

  /**
   * Update the playoff matches.
   * There are a few caveats in this function. 
   * 1. If a tiebreaker is generated, matches may be re-timed in such a way that future matches have past start times. 
   *    The impact of this should be < 1 cycle time, but is a consequence of the retiming process.
   */
  pub fn update(kv: &kv::KVConnection) -> anyhow::Result<()> {
    let alliances = PlayoffAlliance::all_map(kv)?;
    let matches = Match::all(kv)?;
    let playoff_mode = PlayoffMode::get(kv)?;
    let scores = CommittedMatchScores::all_map(kv)?;
    let blocks = ScheduleBlock::sorted(kv)?.into_iter().filter(|x| x.block_type == ScheduleBlockType::Playoff);

    let config = ScoringConfig::get(kv)?;
    let update = match playoff_mode.mode {
      PlayoffModeType::Bracket | PlayoffModeType::DoubleBracket => bracket_update(&playoff_mode, &matches, &scores, config)?
    };

    match update {
      GenerationUpdate::MatchUpdates(updates) => {
        let mut hydrated = vec![];
        for item in updates {
          hydrated.push(HydratedPlayoffScheduleItem { duration: None, item });
        }

        // Assign break times. TODO: This is naiive right now, it doesn't account for award time
        for item in hydrated.iter_mut() {
          match item.item {
            PlayoffScheduleItem::AwardsBreak => item.duration = Some(playoff_mode.minimum_round_break.0.clone()),
            _ => ()
          }
        }

        // Determine how much total time we have available
        let total_time = blocks.clone()
          .map(|b| b.end_time - b.start_time)
          .fold(Duration::zero(), |a, b| a + b);

        // Determine the amount of time per match (cycle time)
        let net_time = total_time - hydrated.iter().filter_map(|x| x.duration).fold(Duration::zero(), |a, b| a + b);
        let mut cycle_time = net_time / (hydrated.iter().filter(|x| matches!(x.item, PlayoffScheduleItem::Match(..))).count() as i32);
        
        // Just in case, set a minimum cycle time of 5 mins
        // TODO: Make this configurable
        if cycle_time.num_seconds() < 5*60 {
          cycle_time = Duration::seconds(5*60);
        }

        // Hydrate the matches
        for item in hydrated.iter_mut() {
          match item.item {
            PlayoffScheduleItem::Match(..) => item.duration = Some(cycle_time.clone()),
            _ => ()
          }
        }

        let mut blocks_mut = blocks.clone();
        let mut this_block = blocks_mut.next();
        let mut offset = this_block.as_ref().map(|x| x.start_time).unwrap_or(chrono::Local::now());

        for item in hydrated.into_iter() {
          // Move to the next block if we've reached the end of the block
          if let Some(block) = this_block.clone() {
            if offset >= block.end_time {
              this_block = blocks_mut.next();
              if let Some(block) = &this_block {
                offset = block.start_time;
              } else {
                warn!("Schedule Overrun!");
              }
            }
          }

          // Update the matches in the database
          // TODO: Also insert breaks into the DB?
          match item.item {
            PlayoffScheduleItem::Match(m) => {
              let (red_teams, red_alliance) = match &m.red {
                PlayoffAllianceDescriptor::Alliance(a) => (alliances.get(a).map(|x| x.teams.clone()).unwrap_or(vec![]), Some(*a)),
                _ => (vec![], None)
              };

              let (blue_teams, blue_alliance) = match &m.blue {
                PlayoffAllianceDescriptor::Alliance(a) => (alliances.get(a).map(|x| x.teams.clone()).unwrap_or(vec![]), Some(*a)),
                _ => (vec![], None)
              };

              let id = Match::gen_id(m.ty, m.round, m.set, m.match_num);
              if let Ok(mut existing) = Match::get(&id, kv) {
                if !existing.played {
                  existing.start_time = offset.clone();
                  existing.red_alliance = red_alliance;
                  existing.blue_alliance = blue_alliance;
                  existing.red_teams = vec![ red_teams.get(0).copied(), red_teams.get(1).copied(), red_teams.get(2).copied() ];
                  existing.blue_teams = vec![ blue_teams.get(0).copied(), blue_teams.get(1).copied(), blue_teams.get(2).copied() ];
                  existing.ready = red_alliance.is_some() && blue_alliance.is_some();
                  existing.insert(kv)?;
                }
              } else {
                let real_match = Match {
                  id,
                  name: Match::gen_name(m.ty, m.round, m.set, m.match_num),
                  start_time: offset.clone(),
                  match_type: m.ty,
                  round: m.round,
                  set_number: m.set,
                  match_number: m.match_num,
                  blue_teams: vec![ blue_teams.get(0).copied(), blue_teams.get(1).copied(), blue_teams.get(2).copied() ],
                  blue_alliance,
                  red_teams: vec![ red_teams.get(0).copied(), red_teams.get(1).copied(), red_teams.get(2).copied() ],
                  red_alliance,
                  dqs: vec![],
                  played: false,
                  ready: blue_alliance.is_some() && red_alliance.is_some()
                };

                real_match.insert(kv)?;
              }
            },
            PlayoffScheduleItem::AwardsBreak | PlayoffScheduleItem::TiebreakerSlot => { }
          }
          offset += item.duration.unwrap();
        }
      },
      GenerationUpdate::TournamentWon { winner, finalist } => {
        info!("Tournament Winner: {}, Finalist: {}", winner, finalist);
        let teams = Team::all_map(kv)?;
        if let Some(winning_alliance) = alliances.get(&winner) {
          let recipients: Vec<AwardRecipient> = winning_alliance.teams.iter().map(|t| AwardRecipient { team: Some(teams.get(t).map(|t| t.display_number.clone()).unwrap_or(t.to_string())), awardee: None }).collect();
          let award = Award { id: "winner".to_owned(), name: "Winner".to_owned(), recipients };
          award.insert(kv)?;
        }

        if let Some(finalist_alliance) = alliances.get(&finalist) {
          let recipients: Vec<AwardRecipient> = finalist_alliance.teams.iter().map(|t| AwardRecipient { team: Some(teams.get(t).map(|t| t.display_number.clone()).unwrap_or(t.to_string())), awardee: None }).collect();
          let award = Award { id: "finalist".to_owned(), name: "Finalist".to_owned(), recipients };
          award.insert(kv)?;
        }
      },
    }
    
    Ok(())
  }
}