use std::collections::HashMap;

use genpdf::{style, elements::{TableLayout, PageBreak}};
use jms_base::kv;
use jms_core_lib::{models::{self, Team}, db::{Singleton, Table}, reports::ReportData};

use super::{pdf_table, report_pdf, render_header};

fn render_team(team: Option<&Option<usize>>, highlight: Option<usize>, alliance: models::Alliance, teams: &HashMap<usize, Team>) -> style::StyledString {
  let team = team.and_then(|t| t.clone());
  let t_string = team.map_or("".to_owned(), |t| teams.get(&t).map(|team| team.display_number.clone()).unwrap_or(format!("{}", t)));

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

fn render_match_table(matches: &Vec<models::Match>, teams: &HashMap<usize, Team>, team_highlight: Option<usize>) -> TableLayout {
  let weights = vec![3, 3, 2, 2, 2, 2, 2, 2];
  let headers = vec!["Time", "Match", "Blue 1", "Blue 2", "Blue 3", "Red 1", "Red 2", "Red 3"];
  let rows: Vec<Vec<style::StyledString>> = matches
    .iter()
    .map(|m| {
      vec![
        style::StyledString::new(
          m.start_time.format("%a %T").to_string(),
          style::Style::new(),
        ),
        style::StyledString::new(m.name.clone(), style::Style::new()),
        render_team(m.blue_teams.get(0), team_highlight, models::Alliance::Blue, teams),
        render_team(m.blue_teams.get(1), team_highlight, models::Alliance::Blue, teams),
        render_team(m.blue_teams.get(2), team_highlight, models::Alliance::Blue, teams),
        render_team(m.red_teams.get(0), team_highlight, models::Alliance::Red, teams),
        render_team(m.red_teams.get(1), team_highlight, models::Alliance::Red, teams),
        render_team(m.red_teams.get(2), team_highlight, models::Alliance::Red, teams),
      ]
    })
    .collect();
  pdf_table(weights, headers, rows)
}

pub fn match_report(mtype: models::MatchType, kv: &kv::KVConnection) -> Result<ReportData, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(kv)?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let matches = models::Match::sorted(kv)?.into_iter().filter(|m| m.match_type == mtype).collect();

  let teams = models::Team::all_map(kv)?;

  let title = format!("{:?} Match Report", mtype);
  let mut doc = report_pdf(&title, &event_name, true);

  doc.push(render_match_table(&matches, &teams, None));
  doc.render(&mut buf)?;

  Ok(ReportData::pdf(buf))
}

pub fn match_report_per_team(mtype: models::MatchType, kv: &kv::KVConnection) -> Result<ReportData, Box<dyn std::error::Error>> {
  let mut buf = vec![];

  let event_details = models::EventDetails::get(kv)?;
  let event_name = event_details.event_name.unwrap_or("Unnamed Event".to_owned());
  let matches = models::Match::sorted(kv)?.into_iter().filter(|m| m.match_type == mtype).collect();

  let teams = models::Team::all_map(kv)?;

  let title = format!("{:?} Match Report (Individual)", mtype);
  let mut doc = report_pdf(&title, &event_name, false);

  for (i, team) in teams.values().enumerate() {
    render_header(
      &mut doc,
      &format!("{:?} Match Report ({})", mtype, team.display_number),
      &event_name,
    );
    doc.push(render_match_table(&matches, &teams, Some(team.number)));
    if i != teams.len() - 1 {
      doc.push(PageBreak::new());
    }
  }

  doc.render(&mut buf)?;

  Ok(ReportData::pdf(buf))
}