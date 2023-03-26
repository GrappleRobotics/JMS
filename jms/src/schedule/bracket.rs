use std::collections::{HashMap, HashSet};

use crate::{schedule::PlayoffAllianceDescriptor, models::{MatchSubtype, PlayoffAlliance, Match, Alliance}};

use super::{IncompleteMatch, GenerationUpdate, create_tiebreaker, playoffs::PlayoffMatchGenerator};

// Note: this must be in order
fn bracket_template(double_elim: bool) -> Vec<IncompleteMatch> {
  if double_elim {
    vec![
      // Round 1
      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(1), blue: PlayoffAllianceDescriptor::Alliance(8), playoff_type: MatchSubtype::Semifinal, set: 1, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(4), blue: PlayoffAllianceDescriptor::Alliance(5), playoff_type: MatchSubtype::Semifinal, set: 2, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(2), blue: PlayoffAllianceDescriptor::Alliance(7), playoff_type: MatchSubtype::Semifinal, set: 3, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(3), blue: PlayoffAllianceDescriptor::Alliance(6), playoff_type: MatchSubtype::Semifinal, set: 4, match_num: 1 },
      // Round 2
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 2), playoff_type: MatchSubtype::Semifinal, set: 7, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 3), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 4), playoff_type: MatchSubtype::Semifinal, set: 8, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::LoserOf(MatchSubtype::Semifinal, 1), blue: PlayoffAllianceDescriptor::LoserOf(MatchSubtype::Semifinal, 2), playoff_type: MatchSubtype::Semifinal, set: 5, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::LoserOf(MatchSubtype::Semifinal, 3), blue: PlayoffAllianceDescriptor::LoserOf(MatchSubtype::Semifinal, 4), playoff_type: MatchSubtype::Semifinal, set: 6, match_num: 1 },
      // Round 3
      IncompleteMatch { red: PlayoffAllianceDescriptor::LoserOf(MatchSubtype::Semifinal, 8), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 5), playoff_type: MatchSubtype::Semifinal, set: 10, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::LoserOf(MatchSubtype::Semifinal, 7), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 6), playoff_type: MatchSubtype::Semifinal, set: 9, match_num: 1 },
      // Round 4
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 7), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 8), playoff_type: MatchSubtype::Semifinal, set: 11, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 10), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 9), playoff_type: MatchSubtype::Semifinal, set: 12, match_num: 1 },
      // Round 5
      IncompleteMatch { red: PlayoffAllianceDescriptor::LoserOf(MatchSubtype::Semifinal, 11), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 12), playoff_type: MatchSubtype::Semifinal, set: 13, match_num: 1 },
      
      // Finals
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 11), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 13), playoff_type: MatchSubtype::Final, set: 1, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 11), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 13), playoff_type: MatchSubtype::Final, set: 1, match_num: 2 },
    ]
  } else {
    vec![
      // Round 1 - Quarters
      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(1), blue: PlayoffAllianceDescriptor::Alliance(8), playoff_type: MatchSubtype::Quarterfinal, set: 1, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(1), blue: PlayoffAllianceDescriptor::Alliance(8), playoff_type: MatchSubtype::Quarterfinal, set: 1, match_num: 2 },

      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(4), blue: PlayoffAllianceDescriptor::Alliance(5), playoff_type: MatchSubtype::Quarterfinal, set: 2, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(4), blue: PlayoffAllianceDescriptor::Alliance(5), playoff_type: MatchSubtype::Quarterfinal, set: 2, match_num: 2 },

      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(2), blue: PlayoffAllianceDescriptor::Alliance(7), playoff_type: MatchSubtype::Quarterfinal, set: 3, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(2), blue: PlayoffAllianceDescriptor::Alliance(7), playoff_type: MatchSubtype::Quarterfinal, set: 3, match_num: 2 },

      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(3), blue: PlayoffAllianceDescriptor::Alliance(6), playoff_type: MatchSubtype::Quarterfinal, set: 4, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::Alliance(3), blue: PlayoffAllianceDescriptor::Alliance(6), playoff_type: MatchSubtype::Quarterfinal, set: 4, match_num: 2 },

      // Round 2 - Semis
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Quarterfinal, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Quarterfinal, 2), playoff_type: MatchSubtype::Semifinal, set: 1, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Quarterfinal, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Quarterfinal, 2), playoff_type: MatchSubtype::Semifinal, set: 1, match_num: 2 },

      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Quarterfinal, 3), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Quarterfinal, 4), playoff_type: MatchSubtype::Semifinal, set: 2, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Quarterfinal, 3), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Quarterfinal, 4), playoff_type: MatchSubtype::Semifinal, set: 2, match_num: 2 },

      // Round 3 - Finals
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 2), playoff_type: MatchSubtype::Final, set: 1, match_num: 1 },
      IncompleteMatch { red: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 1), blue: PlayoffAllianceDescriptor::WinnerOf(MatchSubtype::Semifinal, 2), playoff_type: MatchSubtype::Final, set: 1, match_num: 2 },
    ]
  }
}

pub fn bracket_update(double_elim: bool, alliances: &Vec<PlayoffAlliance>, existing_matches: &Option<Vec<Match>>) -> GenerationUpdate {
  if alliances.len() > 8 {
    panic!("Can't support >8 alliances in brackets!")
  }

  let matches = existing_matches.as_ref().unwrap_or(&vec![]).clone();

  let mut bracket = bracket_template(double_elim);
  
  let mut winners: HashMap<(MatchSubtype, usize), PlayoffAllianceDescriptor> = HashMap::new();
  let mut losers: HashMap<(MatchSubtype, usize), PlayoffAllianceDescriptor> = HashMap::new();
  let mut pending = vec![];
  
  // TODO: Generate Byes
  
  // for bm in bracket.iter_mut() {
    
    // }
    
    // // Remove bye matches
    
  let mut bye_matches = HashSet::new();
  // Generate tiebreakers, assign winners and losers
  for bm in bracket.iter_mut() {
    // Assign byes if appropriate
    if let PlayoffAllianceDescriptor::Alliance(alliance) = bm.blue {
      if alliances.iter().find(|a| a.id == alliance).is_none() {
        bm.blue = PlayoffAllianceDescriptor::Bye;
      }
    }

    if let PlayoffAllianceDescriptor::Alliance(alliance) = bm.red {
      if alliances.iter().find(|a| a.id == alliance).is_none() {
        bm.red = PlayoffAllianceDescriptor::Bye;
      }
    }

    // Assign winners and losers if incomplete (also propagates byes)
    match bm.red {
      PlayoffAllianceDescriptor::WinnerOf(t, s) => {
        if let Some(winner) = winners.get(&(t, s)) {
          bm.red = winner.clone();
        }
      },
      PlayoffAllianceDescriptor::LoserOf(t, s) => {
        if let Some(loser) = losers.get(&(t, s)) {
          bm.red = loser.clone();
        }
      },
      _ => ()
    }

    match bm.blue {
      PlayoffAllianceDescriptor::WinnerOf(t, s) => {
        if let Some(winner) = winners.get(&(t, s)) {
          bm.blue = winner.clone();
        }
      },
      PlayoffAllianceDescriptor::LoserOf(t, s) => {
        if let Some(loser) = losers.get(&(t, s)) {
          bm.blue = loser.clone();
        }
      },
      _ => ()
    }

    // Check for byes - if there's a bye, assign the winner and loser and remove this match
    match (&bm.red, &bm.blue) {
      (PlayoffAllianceDescriptor::Bye, winner) | (winner, PlayoffAllianceDescriptor::Bye) => {
        winners.insert((bm.playoff_type, bm.set), winner.clone());
        losers.insert((bm.playoff_type, bm.set), PlayoffAllianceDescriptor::Bye);
        bye_matches.insert((bm.playoff_type, bm.set));
      },
      _ => ()
    }


    // Calculate tiebreakers, win conditions
    let set_matches = matches.iter().filter(|m| m.match_subtype == Some(bm.playoff_type) && m.set_number == bm.set);
    let played = set_matches.clone().filter(|m| m.played);

    let red_wins = played.clone().filter(|x| x.winner == Some(Alliance::Red)).count();
    let blue_wins = played.clone().filter(|x| x.winner == Some(Alliance::Blue)).count();

    let n_wins_required = match bm.playoff_type {
      MatchSubtype::Final => 2,
      _ if !double_elim => 2,
      _ => 1
    };

    let first_match_in_set = set_matches.clone().next();
    let winner_loser = first_match_in_set.and_then(|m| if red_wins >= n_wins_required {
      Some((m.red_alliance.unwrap(), m.blue_alliance.unwrap()))
    } else if blue_wins >= n_wins_required {
      Some((m.blue_alliance.unwrap(), m.red_alliance.unwrap()))
    } else {
      None
    });

    if played.clone().count() > 0 && winner_loser.is_none() && set_matches.count() - played.clone().count() == 0 {
      // We don't have a winner yet, and there are no outstanding matches
      pending.push(create_tiebreaker(first_match_in_set.unwrap().red_alliance.unwrap(), first_match_in_set.unwrap().blue_alliance.unwrap(), &matches, bm.playoff_type));
    } else if let Some((winner, loser)) = winner_loser {
      if bm.playoff_type == MatchSubtype::Final {
        // The tournament has been won
        return GenerationUpdate::TournamentWon(alliances.iter().find(|&a| a.id == winner).unwrap().clone(), alliances.iter().find(|&a| a.id == loser).unwrap().clone())
      }
      // Store winner and loser of the set
      winners.insert((bm.playoff_type, bm.set), PlayoffAllianceDescriptor::Alliance(winner));
      losers.insert((bm.playoff_type, bm.set), PlayoffAllianceDescriptor::Alliance(loser));
    }
  }

  // Get rid of the bye matches
  bracket.retain(|m| !bye_matches.contains(&(m.playoff_type, m.set)));

  // Add pending tiebreakers
  for m in pending {
    bracket.push(m);
  }

  GenerationUpdate::MatchUpdates(bracket)
}

