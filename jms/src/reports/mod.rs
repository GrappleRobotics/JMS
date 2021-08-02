use chrono::Local;
use genpdf::{Alignment, Document, Element, elements::{self, Paragraph, TableLayout}, fonts, style};

pub mod team_report;
pub mod rankings_report;
pub mod match_report;
pub mod award_report;

// TODO: Embed into binary
pub fn pdf_font() -> fonts::FontFamily<fonts::FontData> {
  fonts::from_files("./fonts", "Helvetica", None).unwrap()
}

pub fn report_pdf(title: &str) -> Document {
  let generated_at = Local::now();
  let generated_at_str = generated_at.format("%a %F %T %z");

  let mut doc = Document::new(pdf_font());

  let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(15);
    decorator.set_header(|page| {
      let mut layout = elements::LinearLayout::vertical();
      if page > 1 {
        layout.push(
            elements::Paragraph::new(format!("Page {}", page)).aligned(Alignment::Center),
        );
      }
      layout.push(elements::Break::new(1));
      layout
  });
  doc.set_page_decorator(decorator);

  doc.set_title(title);

  doc.push(
    elements::Paragraph::new(title)
      .aligned(Alignment::Center)
      .styled(style::Style::new().with_font_size(20))
  );
  doc.push(elements::Break::new(0.5));
  doc.push(
    elements::Paragraph::new(&format!("Generated: {}", generated_at_str))
      .aligned(Alignment::Center)
      .styled(style::Style::new().with_color(style::Color::Greyscale(150u8)))
  );
  doc.push(elements::Break::new(2));

  doc.set_font_size(10);

  doc
}

pub fn pdf_table(header_weights: Vec<usize>, headers: Vec<impl Into<String>>, rows: Vec<Vec<impl Into<String>>>) -> TableLayout {
  let mut table = elements::TableLayout::new(header_weights);
  table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));

  let mut header_row = table.row();
  for head in headers {
    let next = header_row.element(
      Paragraph::new(&head.into())
        .styled(style::Style::new().bold().with_font_size(12))
        .padded(2)
    );
    header_row = next;
  }
  header_row.push().unwrap();

  for r in rows {
    let mut row = table.row();
    for col in r {
      let next = row.element(
        Paragraph::new(&col.into())
          .padded(2)
      );
      row = next;
    }
    row.push().unwrap();
  }

  table
}