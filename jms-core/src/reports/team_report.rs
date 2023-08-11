use jms_base::kv;
use jms_core_lib::{models, db::{Table, Singleton}, reports::ReportData};

use super::{report_pdf, pdf_table};

pub fn teams_report(kv: &kv::KVConnection) -> Result<ReportData, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(kv)?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let teams = models::Team::sorted(kv)?;

  let mut doc = report_pdf("Team Report", &event_name, true);

  let headers = vec!["Team", "D. #", "Name", "Affiliation", "Location", "Sched"];
  let rows: Vec<Vec<String>> = teams
    .iter()
    .map(|t| {
      vec![
        format!("{}", t.number),
        t.display_number.clone(),
        t.name.clone().unwrap_or("".to_owned()),
        t.affiliation.clone().unwrap_or("".to_owned()),
        t.location.clone().unwrap_or("".to_owned()),
        if t.schedule { "TRUE".to_owned() } else { "FALSE".to_owned() }
      ]
    })
    .collect();
  let table = pdf_table(vec![2, 2, 5, 5, 5, 2], headers, rows);

  doc.push(table);
  doc.render(&mut buf)?;

  Ok(ReportData::pdf(buf))
}