use std::collections::{HashMap, HashSet};

use jms_core_lib::{models::{PlayoffAlliance, Match, CommittedMatchScores, Alliance, MatchType, PlayoffMode, PlayoffModeType}, db::Table};

use crate::schedule::playoffs::PlayoffAllianceDescriptor;

use super::playoffs::{IncompleteMatch, GenerationUpdate, PlayoffScheduleItem};

pub const DOUBLE_BRACKET_TEMPLATE: [PlayoffScheduleItem; 21] = [
  // Round 1
  // Upper
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 1, match_num: 1, red: PlayoffAllianceDescriptor::Alliance(1), blue: PlayoffAllianceDescriptor::Alliance(8) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 2, match_num: 1, red: PlayoffAllianceDescriptor::Alliance(4), blue: PlayoffAllianceDescriptor::Alliance(5) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 3, match_num: 1, red: PlayoffAllianceDescriptor::Alliance(2), blue: PlayoffAllianceDescriptor::Alliance(7) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 4, match_num: 1, red: PlayoffAllianceDescriptor::Alliance(3), blue: PlayoffAllianceDescriptor::Alliance(6) }),
  PlayoffScheduleItem::AwardsBreak,
  
  // Round 2
  // Lower
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 2, set: 1, match_num: 1, red: PlayoffAllianceDescriptor::LoserOf(MatchType::Playoff, 1, 1), blue: PlayoffAllianceDescriptor::LoserOf(MatchType::Playoff, 1, 2) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 2, set: 2, match_num: 1, red: PlayoffAllianceDescriptor::LoserOf(MatchType::Playoff, 1, 3), blue: PlayoffAllianceDescriptor::LoserOf(MatchType::Playoff, 1, 4) }),
  // Upper
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 2, set: 3, match_num: 1, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 2) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 2, set: 4, match_num: 1, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 3), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 4) }),
  PlayoffScheduleItem::AwardsBreak,

  // Round 3
  // Lower
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 3, set: 1, match_num: 1, red: PlayoffAllianceDescriptor::LoserOf(MatchType::Playoff, 2, 3), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 2, 2) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 3, set: 2, match_num: 1, red: PlayoffAllianceDescriptor::LoserOf(MatchType::Playoff, 2, 4), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 2, 1) }),
  PlayoffScheduleItem::AwardsBreak,

  // Round 4
  // Upper
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 4, set: 1, match_num: 1, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 2, 3), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 2, 4) }),
  // Lower
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 4, set: 2, match_num: 1, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 3, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 3, 2) }),
  PlayoffScheduleItem::AwardsBreak,

  // Round 5
  // Lower
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 5, set: 1, match_num: 1, red: PlayoffAllianceDescriptor::LoserOf(MatchType::Playoff, 4, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 4, 2) }),
  PlayoffScheduleItem::AwardsBreak,

  // Finals
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Final, round: 1, set: 1, match_num: 1, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 4, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 5, 1) }),
  PlayoffScheduleItem::AwardsBreak,
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Final, round: 1, set: 1, match_num: 2, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 4, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 5, 1) }),
];

pub const SINGLE_BRACKET_TEMPLATE: [PlayoffScheduleItem; 19] = [
  // Round 1 - Quarters
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 1, match_num: 1, red: PlayoffAllianceDescriptor::Alliance(1), blue: PlayoffAllianceDescriptor::Alliance(8) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 2, match_num: 1, red: PlayoffAllianceDescriptor::Alliance(4), blue: PlayoffAllianceDescriptor::Alliance(5) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 3, match_num: 1, red: PlayoffAllianceDescriptor::Alliance(2), blue: PlayoffAllianceDescriptor::Alliance(7) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 4, match_num: 1, red: PlayoffAllianceDescriptor::Alliance(3), blue: PlayoffAllianceDescriptor::Alliance(6) }),
  PlayoffScheduleItem::AwardsBreak,

  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 1, match_num: 2, red: PlayoffAllianceDescriptor::Alliance(1), blue: PlayoffAllianceDescriptor::Alliance(8) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 2, match_num: 2, red: PlayoffAllianceDescriptor::Alliance(4), blue: PlayoffAllianceDescriptor::Alliance(5) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 3, match_num: 2, red: PlayoffAllianceDescriptor::Alliance(2), blue: PlayoffAllianceDescriptor::Alliance(7) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 1, set: 4, match_num: 2, red: PlayoffAllianceDescriptor::Alliance(3), blue: PlayoffAllianceDescriptor::Alliance(6) }),
  PlayoffScheduleItem::AwardsBreak,

  // Round 2 - Semis
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 2, set: 1, match_num: 1, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 2) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 2, set: 2, match_num: 1, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 3), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 4) }),
  PlayoffScheduleItem::AwardsBreak,

  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 2, set: 1, match_num: 2, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 2) }),
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Playoff, round: 2, set: 2, match_num: 2, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 3), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 1, 4) }),
  PlayoffScheduleItem::AwardsBreak,

  // Finals
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Final, round: 1, set: 1, match_num: 1, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 2, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 2, 2) }),
  PlayoffScheduleItem::AwardsBreak,
  PlayoffScheduleItem::Match(IncompleteMatch { ty: MatchType::Final, round: 1, set: 1, match_num: 2, red: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 2, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchType::Playoff, 2, 2) }),
];

// TODO: Single Bracket

pub fn bracket_update(playoff_mode: &PlayoffMode, matches: &Vec<Match>, scores: &HashMap<String, CommittedMatchScores>) -> anyhow::Result<GenerationUpdate> {
  if playoff_mode.n_alliances > 8 {
    anyhow::bail!("Brackets do not currently support >8 alliances!");
  }

  let bracket: &[PlayoffScheduleItem] = match playoff_mode.mode {
    PlayoffModeType::Bracket => &SINGLE_BRACKET_TEMPLATE,
    PlayoffModeType::DoubleBracket => &DOUBLE_BRACKET_TEMPLATE,
  };
  let mut refined_bracket = vec![];   /* Contains tiebreakers, removes byes */

  let mut winners: HashMap<(MatchType, /* round */ usize, /* set */ usize), PlayoffAllianceDescriptor> = HashMap::new();
  let mut losers: HashMap<(MatchType, /* round */ usize, /* set */ usize), PlayoffAllianceDescriptor> = HashMap::new();
  
  for item in bracket.iter() {
    let item = item.clone();
    match item {
      PlayoffScheduleItem::Match(mut incomplete_match) => {
        match incomplete_match.blue {
          // Assign Byes if the alliance does not exist
          PlayoffAllianceDescriptor::Alliance(alliance) if alliance > playoff_mode.n_alliances => incomplete_match.blue = PlayoffAllianceDescriptor::Bye,
          // Assign winners / losers (because we iterate in order, this will also propagate the byes)
          PlayoffAllianceDescriptor::WinnerOf(ty, round, set) => if let Some(descriptor) = winners.get(&(ty, round, set)) {
            incomplete_match.blue = descriptor.clone();
          },
          PlayoffAllianceDescriptor::LoserOf(ty, round, set) => if let Some(descriptor) = losers.get(&(ty, round, set)) {
            incomplete_match.blue = descriptor.clone();
          },
          _ => ()
        }

        match incomplete_match.red {
          // Assign Byes if the alliance does not exist
          PlayoffAllianceDescriptor::Alliance(alliance) if alliance > playoff_mode.n_alliances => incomplete_match.red = PlayoffAllianceDescriptor::Bye,
          // Assign winners / losers (because we iterate in order, this will also propagate the byes)
          PlayoffAllianceDescriptor::WinnerOf(ty, round, set) => if let Some(descriptor) = winners.get(&(ty, round, set)) {
            incomplete_match.red = descriptor.clone();
          },
          PlayoffAllianceDescriptor::LoserOf(ty, round, set) => if let Some(descriptor) = losers.get(&(ty, round, set)) {
            incomplete_match.red = descriptor.clone();
          },
          _ => ()
        }

        // Remove byes, otherwise insert into the refined bracket
        match (&incomplete_match.red, &incomplete_match.blue) {
          (PlayoffAllianceDescriptor::Bye, winner) | (winner, PlayoffAllianceDescriptor::Bye) => {
            winners.insert((incomplete_match.ty, incomplete_match.round, incomplete_match.set), winner.clone());
            losers.insert((incomplete_match.ty, incomplete_match.round, incomplete_match.set), PlayoffAllianceDescriptor::Bye);
          },
          _ => {
            refined_bracket.push(PlayoffScheduleItem::Match(incomplete_match.clone()));
          }
        }

        // Calculate tiebreakers, win conditions
        let set_matches = matches.iter().filter(|m| m.match_type == incomplete_match.ty && m.round == incomplete_match.round && m.set_number == incomplete_match.set);
        let played = set_matches.clone().filter(|m| m.played);

        let match_wins = played.clone().filter_map(|x| scores.get(&x.id()).and_then(|x| x.scores.last()).and_then(|s| s.winner()));

        let red_wins = match_wins.clone().filter(|x| *x == Alliance::Red).count();
        let blue_wins = match_wins.clone().filter(|x| *x == Alliance::Blue).count();

        let n_wins_required = match (&playoff_mode.mode, &incomplete_match.ty) {
          (_, MatchType::Final) => 2,
          (PlayoffModeType::Bracket, _) => 2,
          (PlayoffModeType::DoubleBracket, _) => 1,
        };

        let first_match_in_set = set_matches.clone().next();
        let winner_loser = first_match_in_set.and_then(|m| if red_wins >= n_wins_required {
          Some((m.red_alliance.unwrap(), m.blue_alliance.unwrap()))
        } else if blue_wins >= n_wins_required {
          Some((m.blue_alliance.unwrap(), m.red_alliance.unwrap()))
        } else {
          None
        });

        if played.clone().count() > 0 && winner_loser.is_none() && (set_matches.clone().count() - played.clone().count()) == 0 {
          // We don't have a winner yet, and there are no outstanding matches - queue a tiebreaker
          let max_match_num = played.clone().map(|x| x.match_number).max().unwrap_or(0);
          refined_bracket.push(PlayoffScheduleItem::Match(IncompleteMatch { 
            ty: incomplete_match.ty, round: incomplete_match.round, set: incomplete_match.set, match_num: max_match_num + 1, 
            red: incomplete_match.red, blue: incomplete_match.blue
          }))
        } else if let Some((winner, loser)) = winner_loser {
          if incomplete_match.ty == MatchType::Final {
            // The tournament has been won
            return Ok(GenerationUpdate::TournamentWon { winner, finalist: loser })
          }

          // Store winner and loser of the set
          winners.insert((incomplete_match.ty, incomplete_match.round, incomplete_match.set), PlayoffAllianceDescriptor::Alliance(winner));
          losers.insert((incomplete_match.ty, incomplete_match.round, incomplete_match.set), PlayoffAllianceDescriptor::Alliance(loser));
        }
      },
      PlayoffScheduleItem::AwardsBreak => refined_bracket.push(item),
    }
  }

  Ok(GenerationUpdate::MatchUpdates(refined_bracket))
}
