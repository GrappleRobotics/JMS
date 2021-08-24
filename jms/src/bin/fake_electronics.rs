use std::{io::Cursor, time::Duration};

use cursive::{Vec2, event::Key, theme::{Color, ColorStyle, Effect}, traits::Nameable, views::{Dialog, LinearLayout, Panel}};
use jms::{electronics::protos, models};
use prost::Message;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream, sync::broadcast, time};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut siv = cursive::default();
  let (stop_tx, mut stop_rx) = broadcast::channel(1);

  let mut scoring_table = FakeElectronicsClient::new(protos::NodeRole::NodeScoringTable).await?;
  let mut red_alliance = FakeElectronicsClient::new(protos::NodeRole::NodeRed).await?;
  let mut blue_alliance = FakeElectronicsClient::new(protos::NodeRole::NodeBlue).await?;

  siv.add_layer(
    Dialog::new()
      .title("Fake Electronics")
      .content(
        LinearLayout::vertical()
          .child(Panel::new(ScoringTableView::new().with_name("scoring_table")))
          .child(
            LinearLayout::horizontal()
              .child(Panel::new(AllianceView::new(models::Alliance::Blue).with_name("blue")))
              .child(Panel::new(AllianceView::new(models::Alliance::Red).with_name("red")))
          )

      )
  );

  let mut runner = siv.runner();

  runner.refresh();
  runner.step();

  runner.add_global_callback(Key::Esc, move |_| {
    stop_tx.send(true).unwrap();
  });
  
  let mut interval = time::interval(Duration::from_millis(500));
  loop {
    tokio::select! {
      result = scoring_table.read() => {
        let msg = result?;
        match msg.data.unwrap() {
          protos::update_field2_node::Data::ScoringTable(msg) => {
            let mut st = runner.find_name::<ScoringTableView>("scoring_table").unwrap();
            st.current_state = Some(msg);
          },
          _ => ()
        }
      },
      result = red_alliance.read() => {
        let msg = result?;
        match msg.data.unwrap() {
          protos::update_field2_node::Data::Alliance(msg) => {
            let mut alliance = runner.find_name::<AllianceView>("red").unwrap();
            alliance.current_state = Some(msg)
          },
          _ => ()
        }
      },
      result = blue_alliance.read() => {
        let msg = result?;
        match msg.data.unwrap() {
          protos::update_field2_node::Data::Alliance(msg) => {
            let mut alliance = runner.find_name::<AllianceView>("blue").unwrap();
            alliance.current_state = Some(msg)
          },
          _ => ()
        }
      },
      _ = stop_rx.recv() => {
        return Ok(())
      },
      _ = interval.tick() => ()
    }

    runner.refresh();
    runner.step();
  }
}

// Network Stuff
pub struct FakeElectronicsClient {
  stream: TcpStream,
}

impl FakeElectronicsClient {
  pub async fn new(role: protos::NodeRole) -> anyhow::Result<Self> {
    let msg = protos::UpdateNode2Field {
        ipv4: vec![127, 0, 0, 1],
        role: role.into(),
        data: None,
    };

    let mut s = TcpStream::connect("localhost:5333").await?;
    s.write(&msg.encode_to_vec()).await?;

    Ok(Self {
      stream: s
    })
  }

  pub async fn read(&mut self) -> anyhow::Result<protos::UpdateField2Node> {
    let mut buf = vec![0u8; 256];
    loop {
      let n = self.stream.read(&mut buf).await?;
      if n > 0 {
        let msg = protos::UpdateField2Node::decode(&mut Cursor::new(&buf[0..n]))?;
        return Ok(msg)
      }
    }
  }
}

fn draw_lights(printer: &cursive::Printer, pos: Vec2, len: usize, lights: protos::lights::Mode) {
  let (col, str) = match lights {
    protos::lights::Mode::Off(_) => ( Color::Rgb(0, 0, 0), "OFF" ),
    protos::lights::Mode::Constant(m) => ( Color::Rgb(m.rgb[0], m.rgb[1], m.rgb[2]), "" ),
    protos::lights::Mode::Pulse(m) => ( Color::Rgb(m.rgb[0], m.rgb[1], m.rgb[2]), "PLSE" ),
    protos::lights::Mode::Chase(m) => ( Color::Rgb(m.rgb[0], m.rgb[1], m.rgb[2]), "CHSE" ),
    protos::lights::Mode::Rainbow(_) => ( Color::Rgb(0, 0, 0), "RBOW" ),
  };

  let str = format!("{:width$}", str, width=len);
  printer.with_color(
    ColorStyle::new(Color::Rgb(255, 255, 255), col),
    |printer| printer.print(pos, str.as_str())
  )
}

pub struct ScoringTableView {
  current_state: Option<protos::update_field2_node::ScoringTable>
}

impl ScoringTableView {
  pub fn new() -> Self {
    Self {
      current_state: None
    }
  }
}

impl cursive::view::View for ScoringTableView {
  fn draw(&self, printer: &cursive::Printer) {
    printer.with_effect(Effect::Bold, |printer| printer.print(( 18, 0 ), "Scoring Table"));

    printer.with_color(
      ColorStyle::front(Color::Rgb(0, 0, 255)),
      |printer| {
        printer.print(( 20, 1 ), "BLUE")
      }
    );
    printer.with_color(
      ColorStyle::front(Color::Rgb(255, 0, 0)),
      |printer| {
        printer.print(( 27, 1 ), "RED")
      }
    );

    if let Some(current) = self.current_state.as_ref() {
      let default = protos::lights::Mode::Off(protos::lights::Off {
        off: true
      });
      let red = current.lights1.as_ref().and_then(|l| l.mode.clone()).unwrap_or(default.clone());
      let blue = current.lights2.as_ref().and_then(|l| l.mode.clone()).unwrap_or(default.clone());

      draw_lights(printer, Vec2::new( 3, 1 ), 15, blue);
      draw_lights(printer, Vec2::new( 33, 1 ), 15, red);
    }
  }

  fn required_size(&mut self, _: Vec2) -> Vec2 {
    Vec2::new( 50, 2 )
  }
}

pub struct AllianceView {
  alliance: models::Alliance,
  current_state: Option<protos::update_field2_node::Alliance>
}

impl AllianceView {
  pub fn new(alliance: models::Alliance) -> Self {
    Self {
      alliance,
      current_state: None
    }
  }
}

impl cursive::view::View for AllianceView {
  fn draw(&self, printer: &cursive::Printer) {
    printer.with_effect(
      Effect::Bold, 
      |printer| printer.print(( 7, 0 ), format!("{} Alliance", self.alliance.to_string()).as_str())
    );

    let alliance_color = match self.alliance {
      models::Alliance::Blue => Color::Rgb(0, 0, 255),
      models::Alliance::Red => Color::Rgb(255, 0, 0),
    };

    printer.with_color(
      ColorStyle::front(alliance_color),
      |printer| {
        printer.print(( 2, 1 ), "1");
        printer.print(( 9, 1 ), "2");
        printer.print(( 16, 1 ), "3");
      }
    );

    if let Some(current) = self.current_state.as_ref() {
      let default = protos::lights::Mode::Off(protos::lights::Off {
        off: true
      });
      let one = current.lights1.as_ref().and_then(|l| l.mode.clone()).unwrap_or(default.clone());
      let two = current.lights2.as_ref().and_then(|l| l.mode.clone()).unwrap_or(default.clone());
      let three = current.lights3.as_ref().and_then(|l| l.mode.clone()).unwrap_or(default.clone());

      draw_lights(printer, Vec2::new( 4, 1 ), 4, one);
      draw_lights(printer, Vec2::new( 11, 1 ), 4, two);
      draw_lights(printer, Vec2::new( 18, 1 ), 4, three);
    }
  }
  
  fn required_size(&mut self, _: Vec2) -> Vec2 {
    Vec2::new( 25, 2 )
  }
}