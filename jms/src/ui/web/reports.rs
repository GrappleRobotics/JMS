use rocket::{http::ContentType, response::content::Custom};

use crate::{models, reports::{award_report::awards_report, match_report::match_report, rankings_report::rankings_report, team_report::teams_report, wpa_report::{wpa_report, wpa_report_csv}}};

#[get("/teams")]
pub fn teams() -> Custom<Vec<u8>> {
  Custom(ContentType::PDF, teams_report().unwrap())
}

#[get("/wpa/<format>")]
pub fn wpa(format: String) -> Custom<Vec<u8>> {
  match format.as_str() {
    "csv" => Custom(ContentType::CSV, wpa_report_csv().unwrap()),
    "pdf" => Custom(ContentType::PDF, wpa_report().unwrap()),
    _ => return Custom(ContentType::Text, "Invalid format".as_bytes().to_vec())
  }
}

#[get("/rankings")]
pub fn rankings() -> Custom<Vec<u8>> {
  Custom(ContentType::PDF, rankings_report().unwrap())
}

#[get("/awards")]
pub fn awards() -> Custom<Vec<u8>> {
  Custom(ContentType::PDF, awards_report().unwrap())
}

#[get("/matches/<match_type>")]
pub fn matches(match_type: String) -> Custom<Vec<u8>> {
  let match_type = match match_type.as_str() {
    "quals" => models::MatchType::Qualification,
    "playoffs" => models::MatchType::Playoff,
    _ => return Custom(ContentType::Text, "Invalid match type!".as_bytes().to_vec())
  };
  Custom(ContentType::PDF, match_report(match_type).unwrap())
}