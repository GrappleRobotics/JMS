use crate::{models::{Alliance, Match, MatchSubtype, PlayoffAlliance}, schedule::IncompleteMatch};

use super::{GenerationUpdate, create_tiebreaker};

fn n_sets(level: MatchSubtype) -> usize {
  match level {
    MatchSubtype::Quarterfinal => 4,
    MatchSubtype::Semifinal => 2,
    MatchSubtype::Final => 1,
  }
}

fn pairings(level: MatchSubtype) -> &'static [i32] {
  match level {
    MatchSubtype::Quarterfinal => &[ 1, 8, 4, 5, 2, 7, 3, 6 ],
    MatchSubtype::Semifinal => &[ 1, 4, 2, 3 ],
    MatchSubtype::Final => &[ 1, 2 ],
  }
}

fn fed_by(level: MatchSubtype) -> Option<MatchSubtype> {
  match level {
    MatchSubtype::Quarterfinal => None,
    MatchSubtype::Semifinal => Some(MatchSubtype::Quarterfinal),
    MatchSubtype::Final => Some(MatchSubtype::Semifinal),
  }
}

pub fn bracket_update(alliances: &Vec<PlayoffAlliance>, existing_matches: &Option<Vec<Match>>) -> GenerationUpdate {
  if alliances.len() > 8 {
    panic!("Can't support >8 alliances in brackets!");
  }
  let matches = existing_matches.as_ref().unwrap_or(&vec![]).clone();
  update_bracket_recursive(MatchSubtype::Final, 1, alliances, &matches)
}

fn update_bracket_recursive(bracket: MatchSubtype, set: usize, alliances: &Vec<PlayoffAlliance>, existing: &Vec<Match>) -> GenerationUpdate {
  let sets = n_sets(bracket);

  let mut new_matches = vec![];

  let mut red = None;
  let mut blue = None;

  
  // Fill the bottom brackets
  if alliances.len() < 4*sets {
    // We're either at the bottom, or this round has alliances competing in it for the first time
    // Higher-seeded alliances get promoted first
    let pairs = pairings(bracket);
    let num_direct = 4*sets - alliances.len();
    
    let red_alliance = pairs[ (set-1)*2 ];
    let blue_alliance = pairs[ (set-1)*2 + 1 ];
    
    if red_alliance <= num_direct as i32 {
      red = alliances.iter().find(|&a| a.id == red_alliance).cloned();
    }
    
    if blue_alliance <= num_direct as i32 {
      blue = alliances.iter().find(|&a| a.id == blue_alliance).cloned();
    }
  }

  // Recurse down a bracket to build the schedule
  if let Some(next_level) = fed_by(bracket) {
    if red.is_none() {
      match update_bracket_recursive(next_level, set*2-1, alliances, existing) {
        GenerationUpdate::NoUpdate => (),
        GenerationUpdate::NewMatches(mut nm) => new_matches.append(&mut nm),  // Matches are still being generated
        GenerationUpdate::TournamentWon(winner, _) => red = Some(winner),         // Last bracket was won - insert into this bracket
      };
    }

    if blue.is_none() {
      match update_bracket_recursive(next_level, set*2, alliances, existing) {
        GenerationUpdate::NoUpdate => (),
        GenerationUpdate::NewMatches(mut nm) => new_matches.append(&mut nm),  // Matches are still being generated
        GenerationUpdate::TournamentWon(winner, _) => blue = Some(winner),         // Last bracket was won - insert into this bracket
      };
    }
  }

  // Stagger the matches to make sure back-to-back matches don't happen below the finals level
  new_matches.sort_by(|a, b| {
    n_sets(b.playoff_type).cmp(&n_sets(a.playoff_type))
      .then(a.match_num.cmp(&b.match_num))
      .then(a.set.cmp(&b.set))
  });

  match (red, blue) {
    // Match is ready to be generated and/or update
    (Some(red), Some(blue)) => {
      process_queued_match(bracket, set, red, blue, alliances, existing)
    },
    // This match isn't ready yet
    _ => {
      if new_matches.len() == 0 {
        GenerationUpdate::NoUpdate
      } else {
        GenerationUpdate::NewMatches(new_matches)
      }
    }
  }
}

fn process_queued_match(bracket: MatchSubtype, set: usize, red: PlayoffAlliance, blue: PlayoffAlliance, _alliances: &Vec<PlayoffAlliance>, existing: &Vec<Match>) -> GenerationUpdate {
  let existing_matches = existing.iter().filter(|&m| m.match_subtype == Some(bracket) && m.set_number == set as i32);

  let red_wins = existing_matches.clone().filter(|m| m.winner == Some(Alliance::Red)).count();
  let blue_wins = existing_matches.clone().filter(|m| m.winner == Some(Alliance::Blue)).count();

  if red_wins >= 2 {
    // Red has won this match
    GenerationUpdate::TournamentWon(red, blue)
  } else if blue_wins >= 2 {
    // Blue has won this match
    GenerationUpdate::TournamentWon(blue, red)
  } else if existing_matches.clone().count() == 0 {
    // There are no matches yet - generate the initial set
    GenerationUpdate::NewMatches(vec![
      IncompleteMatch { red: red.id, blue: blue.id, playoff_type: bracket, set: set as i32, match_num: 1 },
      IncompleteMatch { red: red.id, blue: blue.id, playoff_type: bracket, set: set as i32, match_num: 2 },
    ])
  } else if existing_matches.clone().all(|m| m.played) {
    // All have been played, add a tiebreaker to this set
    GenerationUpdate::NewMatches(vec![create_tiebreaker( red.id, blue.id, existing, bracket )])
  } else {
    // Matches are still being played, do nothing
    GenerationUpdate::NoUpdate
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::models::{MatchType, SQLJson};

  #[test]
  #[should_panic]
  fn test_bracket_9() {
    let alliances: Vec<PlayoffAlliance> = (1..=9).map(|i| PlayoffAlliance { id: i, teams: SQLJson(vec![]), ready: true } ).collect();
    bracket_update(&alliances, &None);
  }

  #[test]
  fn test_bracket_8() {
    let playoff_type = MatchSubtype::Quarterfinal;
    let alliances: Vec<PlayoffAlliance> = (1..=8).map(|i| PlayoffAlliance { id: i, teams: SQLJson(vec![]), ready: true } ).collect();
    let results = bracket_update(&alliances, &None);
    
    println!("{:?}", results);
    if let GenerationUpdate::NewMatches(matches) = results {
      assert_eq!(matches.len(), 2*n_sets(playoff_type));
      assert_eq!(matches[0], IncompleteMatch { red: 1, blue: 8, playoff_type, set: 1, match_num: 1 });
      assert_eq!(matches[1], IncompleteMatch { red: 4, blue: 5, playoff_type, set: 2, match_num: 1 });
      assert_eq!(matches[2], IncompleteMatch { red: 2, blue: 7, playoff_type, set: 3, match_num: 1 });
      assert_eq!(matches[3], IncompleteMatch { red: 3, blue: 6, playoff_type, set: 4, match_num: 1 });
      assert_eq!(matches[4], IncompleteMatch { red: 1, blue: 8, playoff_type, set: 1, match_num: 2 });
      assert_eq!(matches[5], IncompleteMatch { red: 4, blue: 5, playoff_type, set: 2, match_num: 2 });
      assert_eq!(matches[6], IncompleteMatch { red: 2, blue: 7, playoff_type, set: 3, match_num: 2 });
      assert_eq!(matches[7], IncompleteMatch { red: 3, blue: 6, playoff_type, set: 4, match_num: 2 });
    } else {
      assert!(false);
    }
  }

  #[test]
  fn test_bracket_5() {
    let playoff_type = MatchSubtype::Semifinal;
    let alliances: Vec<PlayoffAlliance> = (1..=5).map(|i| PlayoffAlliance { id: i, teams: SQLJson(vec![]), ready: true } ).collect();
    let results = bracket_update(&alliances, &None);
    
    println!("{:?}", results);
    if let GenerationUpdate::NewMatches(matches) = results {
      assert_eq!(matches.len(), 4);
      assert_eq!(matches[0], IncompleteMatch { red: 4, blue: 5, playoff_type: MatchSubtype::Quarterfinal, set: 2, match_num: 1 });
      assert_eq!(matches[1], IncompleteMatch { red: 4, blue: 5, playoff_type: MatchSubtype::Quarterfinal, set: 2, match_num: 2 });
      assert_eq!(matches[2], IncompleteMatch { red: 2, blue: 3, playoff_type, set: 2, match_num: 1 });
      assert_eq!(matches[3], IncompleteMatch { red: 2, blue: 3, playoff_type, set: 2, match_num: 2 });
    } else {
      assert!(false);
    }
  }

  #[test]
  fn test_bracket_4() {
    let playoff_type = MatchSubtype::Semifinal;
    let alliances: Vec<PlayoffAlliance> = (1..=4).map(|i| PlayoffAlliance { id: i, teams: SQLJson(vec![]), ready: true } ).collect();
    let results = bracket_update(&alliances, &None);
    
    println!("{:?}", results);
    if let GenerationUpdate::NewMatches(matches) = results {
      assert_eq!(matches.len(), 2*n_sets(playoff_type));
      assert_eq!(matches[0], IncompleteMatch { red: 1, blue: 4, playoff_type, set: 1, match_num: 1 });
      assert_eq!(matches[1], IncompleteMatch { red: 2, blue: 3, playoff_type, set: 2, match_num: 1 });
      assert_eq!(matches[2], IncompleteMatch { red: 1, blue: 4, playoff_type, set: 1, match_num: 2 });
      assert_eq!(matches[3], IncompleteMatch { red: 2, blue: 3, playoff_type, set: 2, match_num: 2 });

      // Tie 1v4
      // 2v3 not ready yet
      let mut existing = vec![
        make_match(&matches[0], true, None),
        make_match(&matches[1], false, None),
        make_match(&matches[2], true, Some(Alliance::Red)),
        make_match(&matches[3], false, None),
      ];

      let results = bracket_update(&alliances, &Some(existing.clone()));

      println!("{:?}", results);
      if let GenerationUpdate::NewMatches(matches) = results {
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], IncompleteMatch { red: 1, blue: 4, playoff_type, set: 1, match_num: 3 });

        // 2v3 - 3 Wins
        existing.push(make_match(&matches[0], false, None));

        existing[1].played = true;
        existing[1].winner = Some(Alliance::Blue);
        existing[3].played = true;
        existing[3].winner = Some(Alliance::Blue);

        // No update - still waiting for the 1v4 tiebreaker
        let results = bracket_update(&alliances, &Some(existing.clone()));
        println!("{:?}", results);
        match results {
          GenerationUpdate::NoUpdate => assert!(true),
          _ => assert!(false)
        }

        // 1v4 - 1 wins
        existing[4].played = true;
        existing[4].winner = Some(Alliance::Red);

        let results = bracket_update(&alliances, &Some(existing.clone()));
        println!("{:?}", results);
        if let GenerationUpdate::NewMatches(matches) = results {
          // Finals time
          assert_eq!(matches.len(), 2);
          assert_eq!(matches[0], IncompleteMatch { red: 1, blue: 3, playoff_type: MatchSubtype::Final, set: 1, match_num: 1 });
          assert_eq!(matches[1], IncompleteMatch { red: 1, blue: 3, playoff_type: MatchSubtype::Final, set: 1, match_num: 2 });
        } else {
          assert!(false);
        }
      } else {
        assert!(false);
      }
    } else {
      assert!(false);
    }
  }

  #[test]
  fn test_bracket_2() {
    let playoff_type = MatchSubtype::Final;
    let alliances: Vec<PlayoffAlliance> = (1..=2).map(|i| PlayoffAlliance { id: i, teams: SQLJson(vec![]), ready: true } ).collect();
    let results = bracket_update(&alliances, &None);
    
    println!("{:?}", results);
    if let GenerationUpdate::NewMatches(matches) = results {

      assert_eq!(matches.len(), 2*n_sets(playoff_type));
      assert_eq!(matches[0], IncompleteMatch { red: 1, blue: 2, playoff_type, set: 1, match_num: 1 });
      assert_eq!(matches[1], IncompleteMatch { red: 1, blue: 2, playoff_type, set: 1, match_num: 2 });

      // Tie match
      let mut existing = vec![
        make_match(&matches[0], true, None),
        make_match(&matches[1], true, Some(Alliance::Red)),
      ];

      let results = bracket_update(&alliances, &Some(existing.clone()));

      println!("{:?}", results);
      if let GenerationUpdate::NewMatches(matches) = results {
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], IncompleteMatch { red: 1, blue: 2, playoff_type, set: 1, match_num: 3 });

        // Win match
        existing.push( make_match(&matches[0], true, Some(Alliance::Red)) );
        let results = bracket_update(&alliances, &Some(existing.clone()));

        println!("{:?}", results);
        if let GenerationUpdate::TournamentWon(winner, finalist) = results {
          assert_eq!(winner.id, 1);
          assert_eq!(finalist.id, 2);
        } else {
          assert!(false);
        }
      } else {
        assert!(false);
      }
    } else {
      assert!(false);
    }
  }

  fn make_match(im: &IncompleteMatch, played: bool, winner: Option<Alliance>) -> Match {
    let mut m = Match::new_test();
    m.match_type = MatchType::Playoff;
    m.match_subtype = Some(im.playoff_type);
    m.set_number = im.set;
    m.match_number = im.match_num;
    m.played = played;
    m.red_alliance = Some(im.red);
    m.blue_alliance = Some(im.blue);
    m.winner = winner;
    m
  }
}