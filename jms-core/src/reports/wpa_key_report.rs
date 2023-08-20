use std::io::Write;

use jms_base::kv;
use jms_core_lib::{reports::ReportData, models, db::{Table, Singleton}};

use super::{report_pdf, pdf_table};

pub fn wpa_report(kv: &kv::KVConnection) -> Result<ReportData, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(kv)?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let teams = models::Team::sorted(kv)?;

  let mut doc = report_pdf("WPA Key Report (FTA ONLY)", &event_name, true);

  let headers = vec!["Team", "Key"];
  let rows: Vec<Vec<String>> = teams
    .iter()
    .map(|t| vec![format!("{} ({})", t.number, t.display_number), t.wpakey.clone()])
    .collect();
  let table = pdf_table(vec![2, 10], headers, rows);

  doc.push(table);
  doc.render(&mut buf)?;

  Ok(ReportData::pdf(buf))
}

pub fn wpa_report_csv(kv: &kv::KVConnection) -> Result<ReportData, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let teams = models::Team::all(kv)?;

  for ref team in teams {
    buf.write(format!("{},{}\n", team.number, team.wpakey).as_bytes())?;
  }

  Ok(ReportData::csv(buf))
}