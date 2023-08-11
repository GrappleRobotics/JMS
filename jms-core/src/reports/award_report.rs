use jms_base::kv;
use jms_core_lib::{models, db::{Table, Singleton}, reports::ReportData};

use super::{report_pdf, pdf_table};

pub fn awards_report(kv: &kv::KVConnection) -> Result<ReportData, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(kv)?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let awards = models::Award::all(kv)?;

  let mut doc = report_pdf("Awards Report", &event_name, true);

  let headers = vec!["Award", "Team", "Awardee"];
  let rows: Vec<Vec<String>> = awards
    .into_iter()
    .flat_map(|a| {
      let name = &a.name;
      a.recipients
        .iter()
        .map(|r| {
          vec![
            name.clone(),
            r.team.as_ref().map_or("".to_owned(), |t| models::Team::display_number_for(t, kv)),
            r.awardee.clone().unwrap_or("".to_owned()),
          ]
        })
        .collect::<Vec<Vec<String>>>()
    })
    .collect();

  let table = pdf_table(vec![3, 1, 5], headers, rows);

  doc.push(table);
  doc.render(&mut buf)?;

  Ok(ReportData::pdf(buf))
}