use crate::models::{Match, MatchSubtype, PlayoffAlliance};

pub mod playoffs;
pub mod quals;
pub mod worker;

pub mod bracket;
pub mod randomiser;
pub mod round_robin;

#[derive(Debug)]
pub enum GenerationUpdate {
  NoUpdate,
  NewMatches(Vec<IncompleteMatch>),
  TournamentWon(PlayoffAlliance, PlayoffAlliance),
}

#[derive(Debug, PartialEq, Eq)]
pub struct IncompleteMatch {
  red: usize,
  blue: usize,
  playoff_type: MatchSubtype,
  set: usize,
  match_num: usize,
}

fn create_tiebreaker(red: usize, blue: usize, matches: &Vec<Match>, playoff_type: MatchSubtype) -> IncompleteMatch {
  let last_match_for_this_pair = matches
    .iter()
    .filter(|&m| {
      m.match_subtype == Some(playoff_type)
        && ((m.blue_alliance == Some(red) && m.red_alliance == Some(blue))
          || (m.red_alliance == Some(red) && m.blue_alliance == Some(blue)))
    })
    .last()
    .unwrap();

  let set = last_match_for_this_pair.set_number;
  // let last_match_in_set = matches.iter().filter(|&m| m.set_number == set).last().unwrap();
  let highest_match_num = matches.iter().filter(|&m| m.set_number == set).map(|m| m.match_number).max();

  IncompleteMatch {
    red,
    blue,
    playoff_type,
    set,
    // match_num: last_match_in_set.match_number + 1,
    match_num: highest_match_num.unwrap() + 1
  }
}
