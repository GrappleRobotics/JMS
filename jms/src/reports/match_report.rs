use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use genpdf::{
  elements::{PageBreak, TableLayout},
  style,
};

use crate::{db::{self, TableType, DBSingleton}, models, reports::{pdf_table, render_header, report_pdf}};

fn render_team(team: Option<&Option<usize>>, highlight: Option<usize>, alliance: models::Alliance) -> style::StyledString {
  let team = team.and_then(|t| t.clone());
  let t_string = team.map_or("".to_owned(), |t| format!("{}", t));

  let text_style = match (team, highlight) {
    (Some(a), Some(b)) if a == b => style::Style::new().bold().with_font_size(12).and(match alliance {
      models::Alliance::Blue => style::Color::Rgb(0, 0, 255),
      models::Alliance::Red => style::Color::Rgb(255, 0, 0),
    }),
    (Some(_), Some(_)) => style::Color::Rgb(85, 85, 85).into(),
    _ => style::Style::new(),
  };

  style::StyledString::new(t_string, text_style)
}

fn to_local(dt: NaiveDateTime) -> DateTime<Local> {
  Local.from_utc_datetime(&dt)
}

fn render_match_table(matches: &Vec<models::Match>, team_highlight: Option<usize>) -> TableLayout {
  let weights = vec![3, 3, 2, 2, 2, 2, 2, 2];
  let headers = vec!["Time", "Match", "Blue 1", "Blue 2", "Blue 3", "Red 1", "Red 2", "Red 3"];
  let rows: Vec<Vec<style::StyledString>> = matches
    .iter()
    .map(|m| {
      vec![
        style::StyledString::new(
          m.start_time
            .as_ref()
            .map(|dt| to_local(dt.0).format("%a %T").to_string())
            .unwrap_or("".to_owned()),
          style::Style::new(),
        ),
        style::StyledString::new(m.name(), style::Style::new()),
        render_team(m.blue_teams.get(0), team_highlight, models::Alliance::Blue),
        render_team(m.blue_teams.get(1), team_highlight, models::Alliance::Blue),
        render_team(m.blue_teams.get(2), team_highlight, models::Alliance::Blue),
        render_team(m.red_teams.get(0), team_highlight, models::Alliance::Red),
        render_team(m.red_teams.get(1), team_highlight, models::Alliance::Red),
        render_team(m.red_teams.get(2), team_highlight, models::Alliance::Red),
      ]
    })
    .collect();
  pdf_table(weights, headers, rows)
}

pub fn match_report(mtype: models::MatchType) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(&db::database())?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let mut matches = models::Match::by_type(mtype, &db::database())?;

  matches.sort();

  let title = format!("{:?} Match Report", mtype);
  let mut doc = report_pdf(&title, &event_name, true);

  doc.push(render_match_table(&matches, None));
  doc.render(&mut buf)?;

  Ok(buf)
}

pub fn match_report_per_team(mtype: models::MatchType) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(&db::database())?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let mut matches = models::Match::by_type(mtype, &db::database())?;

  let teams = models::Team::all(&db::database())?;

  matches.sort();

  let title = format!("{:?} Match Report (Individual)", mtype);
  let mut doc = report_pdf(&title, &event_name, false);

  for (i, team) in teams.iter().enumerate() {
    render_header(
      &mut doc,
      &format!("{:?} Match Report ({})", mtype, team.id),
      &event_name,
    );
    doc.push(render_match_table(&matches, Some(team.id)));
    if i != teams.len() - 1 {
      doc.push(PageBreak::new());
    }
  }

  doc.render(&mut buf)?;

  Ok(buf)
}
