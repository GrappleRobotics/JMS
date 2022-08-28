use std::collections::HashMap;

use log::info;

use crate::{
  models::{Alliance, Match, MatchSubtype, PlayoffAlliance},
  schedule::create_tiebreaker,
};

use super::{GenerationUpdate, IncompleteMatch};

pub fn round_robin_update(alliances: &Vec<PlayoffAlliance>, existing_matches: &Option<Vec<Match>>) -> GenerationUpdate {
  match existing_matches {
    None => GenerationUpdate::MatchUpdates(generate_initial(alliances)),
    Some(matches) => {
      if matches.len() == 0 {
        GenerationUpdate::MatchUpdates(generate_initial(alliances))
      } else if matches.iter().all(|m| m.played) {
        // May require updating
        if matches.iter().any(|m| m.match_subtype.unwrap() == MatchSubtype::Final) {
          // Update finals - tiebreaker / event end conditions
          let final_matches: Vec<&Match> = matches
            .iter()
            .filter(|m| m.match_subtype == Some(MatchSubtype::Final))
            .collect();
          let red_alliance = alliances
            .iter()
            .find(|a| a.id == final_matches[0].red_alliance.unwrap())
            .unwrap();
          let blue_alliance = alliances
            .iter()
            .find(|a| a.id == final_matches[0].blue_alliance.unwrap())
            .unwrap();

          let red_wins = final_matches.iter().filter(|m| m.winner == Some(Alliance::Red)).count();
          let blue_wins = final_matches
            .iter()
            .filter(|m| m.winner == Some(Alliance::Blue))
            .count();

          if red_wins >= 2 {
            GenerationUpdate::TournamentWon(red_alliance.clone(), blue_alliance.clone())
          } else if blue_wins >= 2 {
            GenerationUpdate::TournamentWon(blue_alliance.clone(), red_alliance.clone())
          } else {
            GenerationUpdate::MatchUpdates(vec![create_tiebreaker(
              red_alliance.id,
              blue_alliance.id,
              matches,
              MatchSubtype::Final,
            )])
          }
        } else {
          // Still in RR Semis
          let standings = standings(&matches);
          // TODO: Don't need a tiebreaker for 1st place - they'll both be in finals regardless
          // if standings[0].1 == standings[1].1 {
          //   // Need a tiebreaker
          //   GenerationUpdate::MatchUpdates(vec![create_tiebreaker(
          //     standings[0].0,
          //     standings[1].0,
          //     matches,
          //     MatchSubtype::Semifinal,
          //   )])
          // } else if standings[1].1 == standings[2].1 {
          if standings[1].1 == standings[2].1 {
            // Need a tiebreaker for 2nd
            GenerationUpdate::MatchUpdates(vec![create_tiebreaker(
              standings[1].0,
              standings[2].0,
              matches,
              MatchSubtype::Semifinal,
            )])
          } else {
            // Generate finals, nice and easy
            GenerationUpdate::MatchUpdates(vec![
              IncompleteMatch {
                red: Some(standings[0].0),
                blue: Some(standings[1].0),
                playoff_type: MatchSubtype::Final,
                set: 1,
                match_num: 1,
              },
              IncompleteMatch {
                red: Some(standings[0].0),
                blue: Some(standings[1].0),
                playoff_type: MatchSubtype::Final,
                set: 1,
                match_num: 2,
              },
            ])
          }
        }
      } else {
        // Still matches to go - don't update yet
        GenerationUpdate::MatchUpdates(vec![])
      }
    }
  }
}

fn generate_initial(alliances: &Vec<PlayoffAlliance>) -> Vec<IncompleteMatch> {
  let mut match_pairings = vec![];

  let mut alliances: Vec<Option<&PlayoffAlliance>> = alliances.iter().map(|a| Some(a)).collect();
  if alliances.len() % 2 != 0 {
    // Add a "Bye"
    alliances.push(None);
  }

  let n = alliances.len();
  let rounds = n - 1;
  let matches_per_round = n / 2;

  let mut alliance_map: Vec<usize> = (1..n).collect();

  // Generate the rounds
  for round in 0..rounds {
    let mut fixed_alliance_map = alliance_map.clone();
    fixed_alliance_map.insert(0, 0);

    let high = &fixed_alliance_map[0..matches_per_round];
    let low = &fixed_alliance_map[matches_per_round..n];

    for i in 0..matches_per_round {
      let a = alliances[high[i]];
      let b = alliances[low[matches_per_round - 1 - i]];

      match (a, b) {
        (Some(a), Some(b)) => {
          match_pairings.push(IncompleteMatch {
            red: Some(a.id),
            blue: Some(b.id),
            playoff_type: MatchSubtype::Semifinal,
            set: i + 1,
            match_num: round + 1,
          });
        }
        (a, b) => {
          info!("RR BYE ({}:{}): {:?} {:?}", round, i, a.map(|x| x.id), b.map(|x| x.id));
        }
      }
    }

    // Rotate the remaining indices
    let popped = alliance_map.pop();
    alliance_map.insert(0, popped.unwrap());
  }

  match_pairings
}

fn standings(matches: &Vec<Match>) -> Vec<(usize, usize)> /* (Alliance, Score) */ {
  let mut alliance_scores = HashMap::new();
  for m in matches {
    let red = m.red_alliance.unwrap();
    let blue = m.blue_alliance.unwrap();

    let (blue_points, red_points) = match m.winner {
      Some(ref winner) => match winner {
        Alliance::Blue => (2, 0),
        Alliance::Red => (0, 2),
      },
      None => (1, 1),
    };

    for (alliance, points) in vec![(red, red_points), (blue, blue_points)] {
      if !alliance_scores.contains_key(&alliance) {
        alliance_scores.insert(alliance, 0);
      }
      *alliance_scores.get_mut(&alliance).unwrap() += points;
    }
  }

  // Sort by score first, then alliance number
  let mut score_vec: Vec<(usize, usize)> = alliance_scores.iter().map(|p| (*p.0, *p.1)).collect();
  score_vec.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
  score_vec
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use crate::models::MatchType;

//   #[test]
//   fn test_rr_initial() {
//     let alliances = vec![
//       PlayoffAlliance {
//         id: 1,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 2,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 3,
//         teams: vec![],
//         ready: true,
//       },
//     ];

//     let results = round_robin_update(&alliances, &None);

//     if let GenerationUpdate::NewMatches(results) = results {
//       let mut vs_matrix = [[0 as usize; 3]; 3];
//       for imatch in &results {
//         vs_matrix[(imatch.red - 1) as usize][(imatch.blue - 1) as usize] += 1;
//         vs_matrix[(imatch.blue - 1) as usize][(imatch.red - 1) as usize] += 1;
//       }

//       println!("{:?}", results);
//       assert_eq!(vs_matrix, [[0, 1, 1], [1, 0, 1], [1, 1, 0]]);
//     } else {
//       assert!(false);
//     }
//   }

//   #[test]
//   fn test_rr_in_progress() {
//     let alliances = vec![
//       PlayoffAlliance {
//         id: 1,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 2,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 3,
//         teams: vec![],
//         ready: true,
//       },
//     ];

//     let matches = vec![
//       make_rr_semi_match(1, 1, true, 1, 2, Some(Alliance::Blue)),
//       make_rr_semi_match(2, 1, false, 3, 2, Some(Alliance::Red)),
//       make_rr_semi_match(3, 1, false, 3, 1, None),
//     ];

//     let results = round_robin_update(&alliances, &Some(matches));
//     if let GenerationUpdate::NoUpdate = results {
//       assert!(true);
//     } else {
//       assert!(false);
//     }
//   }

//   #[test]
//   fn test_rr_standings() {
//     let matches = vec![
//       // 1 v 2, 2 wins
//       make_rr_semi_match(1, 1, true, 2, 1, Some(Alliance::Blue)),
//       // 2 v 3, 1 wins
//       make_rr_semi_match(2, 1, true, 3, 1, Some(Alliance::Red)),
//       // 2 v 3, tie
//       make_rr_semi_match(3, 1, true, 3, 2, None),
//     ];

//     // Final scores should be: 2: 1-0-1 (3pts), 1: 1-1-0 (2pt), 3: 0-1-1 (1pts)
//     let standings = standings(&matches);

//     assert_eq!(standings, vec![(2, 3), (1, 2), (3, 1)]);
//   }

//   #[test]
//   fn test_rr_tiebreaker() {
//     let alliances = vec![
//       PlayoffAlliance {
//         id: 1,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 2,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 3,
//         teams: vec![],
//         ready: true,
//       },
//     ];

//     let matches = vec![
//       // 1 v 2, 1 wins
//       make_rr_semi_match(1, 1, true, 1, 2, Some(Alliance::Blue)),
//       // 2 v 3, 2 wins
//       make_rr_semi_match(2, 1, true, 2, 3, Some(Alliance::Blue)),
//       // 1 v 3, 3 wins
//       make_rr_semi_match(3, 1, true, 1, 3, Some(Alliance::Red)),
//     ];

//     // Tiebreaker - circle of death

//     let new_matches = round_robin_update(&alliances, &Some(matches));

//     if let GenerationUpdate::NewMatches(new_matches) = new_matches {
//       assert_eq!(new_matches.len(), 1);

//       assert_eq!(new_matches[0].red, 1);
//       assert_eq!(new_matches[0].blue, 2);
//       assert_eq!(new_matches[0].playoff_type, MatchSubtype::Semifinal);
//       assert_eq!(new_matches[0].set, 1);
//       assert_eq!(new_matches[0].match_num, 2);
//     } else {
//       assert!(false);
//     }
//   }

//   #[test]
//   fn test_rr_tiebreaker_2nd() {
//     let alliances = vec![
//       PlayoffAlliance {
//         id: 1,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 2,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 3,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 4,
//         teams: vec![],
//         ready: true,
//       },
//     ];

//     let matches = vec![
//       // Set 1
//       make_rr_semi_match(1, 1, true, 1, 2, Some(Alliance::Blue)),
//       make_rr_semi_match(1, 2, true, 3, 4, Some(Alliance::Blue)),
//       // Set 2
//       make_rr_semi_match(2, 1, true, 2, 3, None),
//       make_rr_semi_match(2, 2, true, 1, 4, Some(Alliance::Blue)),
//       // Set 3
//       make_rr_semi_match(3, 1, true, 1, 3, Some(Alliance::Blue)),
//       make_rr_semi_match(3, 2, true, 4, 2, Some(Alliance::Red)),
//     ];

//     // Alliance 1 wins outright 3-0-0
//     // Alliance 2 & 3 both tie for 2nd (1-1-1) - tiebreaker is created between these two
//     // Alliance 4 loses 0-0-3

//     let new_matches = round_robin_update(&alliances, &Some(matches));

//     if let GenerationUpdate::NewMatches(new_matches) = new_matches {
//       assert_eq!(new_matches.len(), 1);

//       assert_eq!(new_matches[0].red, 2);
//       assert_eq!(new_matches[0].blue, 3);
//       assert_eq!(new_matches[0].playoff_type, MatchSubtype::Semifinal);
//       assert_eq!(new_matches[0].set, 2);
//       assert_eq!(new_matches[0].match_num, 3);
//     } else {
//       assert!(false);
//     }
//   }

//   #[test]
//   fn test_rr_final_gen() {
//     let alliances = vec![
//       PlayoffAlliance {
//         id: 1,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 2,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 3,
//         teams: vec![],
//         ready: true,
//       },
//     ];

//     let matches = vec![
//       // 1 v 2, 1 wins
//       make_rr_semi_match(1, 1, true, 1, 2, Some(Alliance::Blue)),
//       // 2 v 3, 2 wins
//       make_rr_semi_match(2, 1, true, 2, 3, Some(Alliance::Blue)),
//       // 1 v 3, 1 wins
//       make_rr_semi_match(3, 1, true, 1, 3, Some(Alliance::Blue)),
//     ];

//     let new_matches = round_robin_update(&alliances, &Some(matches));

//     if let GenerationUpdate::NewMatches(new_matches) = new_matches {
//       assert_eq!(new_matches.len(), 2);
//       assert_eq!(new_matches[0].playoff_type, MatchSubtype::Final);
//       assert_eq!(new_matches[1].playoff_type, MatchSubtype::Final);
//       assert_eq!(new_matches[0].red, 1);
//       assert_eq!(new_matches[1].red, 1);
//       assert_eq!(new_matches[0].blue, 2);
//       assert_eq!(new_matches[1].blue, 2);
//       assert_eq!(new_matches[0].set, 1);
//       assert_eq!(new_matches[1].set, 1);
//       assert_eq!(new_matches[0].match_num, 1);
//       assert_eq!(new_matches[1].match_num, 2);
//     } else {
//       assert!(false);
//     }
//   }

//   #[test]
//   fn test_rr_final_tiebreaker() {
//     let alliances = vec![
//       PlayoffAlliance {
//         id: 1,
//         teams: vec![],
//         ready: true,
//       },
//       PlayoffAlliance {
//         id: 2,
//         teams: vec![],
//         ready: true,
//       },
//     ];

//     let matches = vec![
//       make_final_match(1, true, 1, 2, Some(Alliance::Red)),
//       make_final_match(2, true, 1, 2, Some(Alliance::Blue)),
//       make_final_match(3, true, 1, 2, None),
//     ];

//     let new_matches = round_robin_update(&alliances, &Some(matches));

//     if let GenerationUpdate::NewMatches(new_matches) = new_matches {
//       assert_eq!(new_matches.len(), 1);
//       assert_eq!(new_matches[0].set, 1);
//       assert_eq!(new_matches[0].match_num, 4);
//       assert_eq!(new_matches[0].blue, 1);
//       assert_eq!(new_matches[0].red, 2);
//       assert_eq!(new_matches[0].playoff_type, MatchSubtype::Final);
//     } else {
//       assert!(false);
//     }
//   }

//   fn make_rr_semi_match(
//     set: usize,
//     num: usize,
//     played: bool,
//     blue_alliance: usize,
//     red_alliance: usize,
//     winner: Option<Alliance>,
//   ) -> Match {
//     let mut m = Match::new_test();
//     m.set_number = set;
//     m.match_number = num;
//     m.match_type = MatchType::Playoff;
//     m.match_subtype = Some(MatchSubtype::Semifinal);
//     m.played = played;
//     m.red_alliance = Some(red_alliance);
//     m.blue_alliance = Some(blue_alliance);
//     m.winner = winner;
//     m
//   }

//   fn make_final_match(num: usize, played: bool, blue: usize, red: usize, winner: Option<Alliance>) -> Match {
//     let mut m = Match::new_test();
//     m.set_number = 1;
//     m.match_number = num;
//     m.match_type = MatchType::Playoff;
//     m.match_subtype = Some(MatchSubtype::Final);
//     m.played = played;
//     m.red_alliance = Some(red);
//     m.blue_alliance = Some(blue);
//     m.winner = winner;
//     m
//   }
// }
