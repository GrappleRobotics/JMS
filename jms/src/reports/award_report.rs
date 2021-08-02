use diesel::RunQueryDsl;

use crate::{db, models, reports::{pdf_table, report_pdf}};

pub fn awards_report() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(&db::connection())?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let awards = {
    use crate::schema::awards::dsl::*;
    awards.load::<models::Award>(&db::connection())?
  };
  
  let title = format!("Awards Report - {}", event_name);
  let mut doc = report_pdf(&title);
  
  let headers = vec!["Award", "Team", "Awardee"];
  let rows: Vec<Vec<String>> = awards.iter().flat_map(|a| {
    let name = &a.name;
    a.recipients.0.iter().map(|r| {
      vec![
        name.clone(),
        r.team.map_or("".to_owned(), |t| format!("{}", t)),
        r.awardee.clone().unwrap_or("".to_owned()),
      ]
    }).collect::<Vec<Vec<String>>>()
  }).collect();

  let table = pdf_table(vec![3, 1, 5], headers, rows);

  doc.push(table);
  doc.render(&mut buf)?;

  Ok(buf)
}