use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

use crate::{db, models, reports::{pdf_table, report_pdf}};

fn render_team(team: Option<&Option<i32>>) -> String {
  team.and_then(|t| t.clone()).map_or("".to_owned(), |t| format!("{}", t))
}

fn to_local(dt: NaiveDateTime) -> DateTime<Local> {
  Local.from_utc_datetime(&dt)
}

pub fn match_report(mtype: models::MatchType) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(&db::connection())?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let mut matches = models::Match::with_type(mtype);

  matches.sort();
  
  let title = format!("{:?} Match Report - {}", mtype, event_name);
  let mut doc = report_pdf(&title);
  
  let weights = vec![3, 3, 2, 2, 2, 2, 2, 2];
  let headers = vec!["Time", "Match", "Blue 1", "Blue 2", "Blue 3", "Red 1", "Red 2", "Red 3"];
  let rows: Vec<Vec<String>> = matches.iter().map(|m| {
    vec![
      m.start_time.as_ref().map(|dt| to_local(dt.0).format("%a %T").to_string()).unwrap_or("".to_owned()),
      m.name(),
      render_team(m.blue_teams.0.get(0)),
      render_team(m.blue_teams.0.get(1)),
      render_team(m.blue_teams.0.get(2)),
      render_team(m.red_teams.0.get(0)),
      render_team(m.red_teams.0.get(1)),
      render_team(m.red_teams.0.get(2)),
    ]
  }).collect();
  let table = pdf_table(weights, headers, rows);

  doc.push(table);
  doc.render(&mut buf)?;

  Ok(buf)
}