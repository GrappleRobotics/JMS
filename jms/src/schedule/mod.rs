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
  red: i32,
  blue: i32,
  playoff_type: MatchSubtype,
  set: i32,
  match_num: i32,
}

fn create_tiebreaker(red: i32, blue: i32, matches: &Vec<Match>, playoff_type: MatchSubtype) -> IncompleteMatch {
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
  let last_match_in_set = matches.iter().filter(|&m| m.set_number == set).last().unwrap();

  IncompleteMatch {
    red,
    blue,
    playoff_type,
    set,
    match_num: last_match_in_set.match_number + 1,
  }
}
