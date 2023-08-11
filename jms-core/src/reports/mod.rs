pub mod award_report;
pub mod match_report;
pub mod rankings_report;
pub mod team_report;
pub mod wpa_key_report;

pub mod service;

use std::io::Write;

use chrono::Local;
use genpdf::{
  elements::{self, Paragraph, TableLayout},
  fonts, style, Alignment, Document, Element,
};
use log::info;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "fonts"]
struct Fonts;

pub fn pdf_font() -> fonts::FontFamily<fonts::FontData> {
  let font_path = std::path::Path::new("/tmp/jms/fonts");
  if !font_path.exists() {
    info!("Extracting Fonts.");
    std::fs::create_dir_all(font_path).unwrap();
    for filename in Fonts::iter() {
      let content = Fonts::get(&filename).unwrap();
      let fpath = font_path.join(&*filename);
      std::fs::File::create(fpath).unwrap().write_all(&content.data).unwrap();
    }
    info!("Fonts Extracted.");
  }

  fonts::from_files(font_path.to_str().unwrap(), "Helvetica", None).unwrap()
}

pub fn render_header(doc: &mut Document, title: &str, subtitle: &str) {
  let generated_at = Local::now();
  let generated_at_str = generated_at.format("%a %F %T %z");

  doc.push(
    elements::Paragraph::new(title)
      .aligned(Alignment::Center)
      .styled(style::Style::new().with_font_size(20)),
  );
  doc.push(elements::Break::new(0.25));
  doc.push(
    elements::Paragraph::new(subtitle)
      .aligned(Alignment::Center)
      .styled(style::Style::new().with_font_size(14)),
  );
  doc.push(elements::Break::new(0.5));
  doc.push(
    elements::Paragraph::new(&format!("Generated: {}", generated_at_str))
      .aligned(Alignment::Center)
      .styled(style::Style::new().with_color(style::Color::Greyscale(150u8))),
  );
  doc.push(elements::Break::new(2));
}

pub fn report_pdf(title: &str, subtitle: &str, content: bool) -> Document {
  let mut doc = Document::new(pdf_font());

  let mut decorator = genpdf::SimplePageDecorator::new();
  decorator.set_margins(15);
  decorator.set_header(|_page| {
    let layout = elements::LinearLayout::vertical();
    layout
  });
  doc.set_page_decorator(decorator);

  doc.set_title(format!("{} - {}", title, subtitle).as_str());

  if content {
    render_header(&mut doc, title, subtitle);
  }

  doc.set_font_size(10);

  doc
}

pub fn pdf_table(
  header_weights: Vec<usize>,
  headers: Vec<impl Into<style::StyledString>>,
  rows: Vec<Vec<impl Into<style::StyledString>>>,
) -> TableLayout {
  let mut table = elements::TableLayout::new(header_weights);
  table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

  let mut header_row = table.row();
  for head in headers {
    let next = header_row.element(
      Paragraph::new(head)
        .styled(style::Style::new().bold().with_font_size(12))
        .padded(2),
    );
    header_row = next;
  }
  header_row.push().unwrap();

  for r in rows {
    let mut row = table.row();
    for col in r {
      let next = row.element(Paragraph::new(col).padded(2));
      row = next;
    }
    row.push().unwrap();
  }

  table
}