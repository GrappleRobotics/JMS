use crate::{db::{self, TableType, DBSingleton}, models, reports::{pdf_table, report_pdf}};

pub fn teams_report() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(&db::database())?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let teams = models::Team::all(&db::database())?;

  let mut doc = report_pdf("Team Report", &event_name, true);

  let headers = vec!["Team", "Name", "Affiliation", "Location", "Sched"];
  let rows: Vec<Vec<String>> = teams
    .iter()
    .map(|t| {
      vec![
        format!("{}", t.id),
        t.name.clone().unwrap_or("".to_owned()),
        t.affiliation.clone().unwrap_or("".to_owned()),
        t.location.clone().unwrap_or("".to_owned()),
        if t.schedule { "TRUE".to_owned() } else { "FALSE".to_owned() }
      ]
    })
    .collect();
  let table = pdf_table(vec![2, 5, 5, 5, 2], headers, rows);

  doc.push(table);
  doc.render(&mut buf)?;

  Ok(buf)
}
