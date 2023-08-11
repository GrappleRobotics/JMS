use jms_base::kv;
use jms_core_lib::{models::{self, Team}, db::Singleton, reports::ReportData};

use super::{report_pdf, pdf_table};

pub fn rankings_report(kv: &kv::KVConnection) -> Result<ReportData, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(kv)?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let rankings = models::TeamRanking::sorted(kv)?;

  let mut doc = report_pdf("Qualification Rankings Report", &event_name, true);

  let weights = vec![3, 3, 3, 3, 3, 5, 3, 3];
  let headers = vec!["Rank", "Team", "Played", "RP", "Auto", "Endgame", "Teleop", "W-L-T"];
  let rows: Vec<Vec<String>> = rankings
    .iter()
    .enumerate()
    .map(|(i, r)| {
      vec![
        format!("{}", i + 1),
        Team::display_number(r.team, kv),
        format!("{}", r.played),
        format!("{}", r.rp),
        format!("{}", r.auto_points),
        format!("{}", r.endgame_points),
        format!("{}", r.teleop_points),
        format!("{}-{}-{}", r.win, r.loss, r.tie),
      ]
    })
    .collect();
  let table = pdf_table(weights, headers, rows);

  doc.push(table);
  doc.render(&mut buf)?;

  Ok(ReportData::pdf(buf))
}