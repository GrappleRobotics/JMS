use std::io::Write;

use crate::{db::{self, TableType, DBSingleton}, models, reports::{pdf_table, report_pdf}};

pub fn wpa_report() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(&db::database())?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let teams = models::Team::all(&db::database())?;

  let mut doc = report_pdf("WPA Key Report (FTA ONLY)", &event_name, true);

  let headers = vec!["Team", "Key"];
  let rows: Vec<Vec<String>> = teams
    .iter()
    .map(|t| vec![format!("{}", t.id), t.wpakey.clone().unwrap_or("<No Key>".to_owned())])
    .collect();
  let table = pdf_table(vec![2, 10], headers, rows);

  doc.push(table);
  doc.render(&mut buf)?;

  Ok(buf)
}

pub fn wpa_report_csv() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let teams = models::Team::all(&db::database())?;

  for ref team in teams {
    buf.write(format!("{},{}\n", team.id, team.wpakey.as_ref().map_or("", |s| s.as_str())).as_bytes())?;
  }

  Ok(buf)
}
