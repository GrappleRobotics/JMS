use crate::{db, models, reports::{pdf_table, report_pdf}};

pub fn rankings_report() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(&db::connection())?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let rankings = models::TeamRanking::get_sorted(&db::connection())?;
  
  let title = format!("Qualification Rankings Report - {}", event_name);
  let mut doc = report_pdf(&title);
  
  let weights = vec![3, 3, 3, 3, 3, 5, 3, 3];
  let headers = vec!["Rank", "Team", "Played", "RP", "Auto", "Endgame", "Teleop", "W-L-T"];
  let rows: Vec<Vec<String>> = rankings.iter().enumerate().map(|(i, r)| {
    vec![
      format!("{}", i + 1),
      format!("{}", r.team),
      format!("{}", r.played),
      format!("{}", r.rp),
      format!("{}", r.auto_points),
      format!("{}", r.endgame_points),
      format!("{}", r.teleop_points),
      format!("{}-{}-{}", r.win, r.loss, r.tie),
    ]
  }).collect();
  let table = pdf_table(weights, headers, rows);

  doc.push(table);
  doc.render(&mut buf)?;

  Ok(buf)
}