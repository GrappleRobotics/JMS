use rocket::http::ContentType;

use crate::{
  models,
  reports::{
    award_report::awards_report,
    match_report::{match_report, match_report_per_team},
    rankings_report::rankings_report,
    team_report::teams_report,
    wpa_report::{wpa_report, wpa_report_csv},
  },
};

type RawContentResponse = (ContentType, Vec<u8>);

#[get("/teams")]
pub fn teams() -> RawContentResponse {
  (ContentType::PDF, teams_report().unwrap())
}

#[get("/wpa/<format>")]
pub fn wpa(format: String) -> RawContentResponse {
  match format.as_str() {
    "csv" => (ContentType::CSV, wpa_report_csv().unwrap()),
    "pdf" => (ContentType::PDF, wpa_report().unwrap()),
    _ => return (ContentType::Text, "Invalid format".as_bytes().to_vec()),
  }
}

#[get("/rankings")]
pub fn rankings() -> RawContentResponse {
  (ContentType::PDF, rankings_report().unwrap())
}

#[get("/awards")]
pub fn awards() -> RawContentResponse {
  (ContentType::PDF, awards_report().unwrap())
}

#[get("/matches/<match_type>/individual")]
pub fn matches_per_team(match_type: String) -> RawContentResponse {
  let match_type = match match_type.as_str() {
    "quals" => models::MatchType::Qualification,
    "playoffs" => models::MatchType::Playoff,
    _ => return (ContentType::Text, "Invalid match type!".as_bytes().to_vec()),
  };
  (ContentType::PDF, match_report_per_team(match_type).unwrap())
}

#[get("/matches/<match_type>")]
pub fn matches(match_type: String) -> RawContentResponse {
  let match_type = match match_type.as_str() {
    "quals" => models::MatchType::Qualification,
    "playoffs" => models::MatchType::Playoff,
    _ => return (ContentType::Text, "Invalid match type!".as_bytes().to_vec()),
  };
  (ContentType::PDF, match_report(match_type).unwrap())
}
