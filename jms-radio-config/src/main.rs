mod imager;

use clap::Parser;
use imager::image;
use jms_util::{net::{self, LinkMetadata}, WPAKeys};
use tokio::{net::TcpStream, io::AsyncReadExt};

#[derive(Parser, Debug)]
struct Args {
  /// The interface to run the imager on. If not provided, will prompt.
  #[clap(short, long, value_parser)]
  iface: Option<String>,
  /// The CSV file of keys. If not provided, will query JMS at 10.0.100.5
  #[clap(short, long, value_parser)]
  keys: Option<String>,
  /// Print the keys and exit
  #[clap(short, long, action)]
  show_keys: bool,
  /// The team to image. If not provided, will run in interactive mode
  #[clap(value_parser)]
  team: Option<u16>,
}

#[derive(serde::Deserialize)]
struct CSVLine {
  team: u16,
  key: String
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let handle = net::handle()?;
  let valid_ifaces = net::get_all_ifaces(&handle).await?;
  let mut all_keys: WPAKeys = WPAKeys::new();

  let args = Args::parse();
  let iface = args.iface.and_then(|i| valid_ifaces.iter().find(|iface| iface.name == i));
  let iface = iface.cloned().unwrap_or_else(|| {
    inquire::Select::<LinkMetadata>::new("Select Interface", valid_ifaces).prompt().unwrap()
  });

  match args.keys {
    Some(keyfile) => {
      // Load from CSV
      let mut reader = csv::ReaderBuilder::new().has_headers(false).from_path(&keyfile)?;
      for result in reader.deserialize() {
        let record: CSVLine = result?;
        all_keys.insert(record.team, record.key);
      }
    },
    None => {
      // Load from JMS
      let mut stream = TcpStream::connect("10.0.100.5:6789").await?;
      let len = stream.read_u32().await? as usize;

      let mut buf = vec![0; len];
      stream.read_exact(&mut buf).await?;

      all_keys = serde_json::from_slice(&buf)?;
    },
  };

  if args.show_keys {
    println!("===== WPA Keys Start =====");
    for (team, key) in all_keys.iter() {
      println!("{}: {}", team, key);
    }
    println!("====== WPA Keys End ======");
    return Ok(())
  }

  match args.team {
    Some(team) => {
      if let Some(key) = all_keys.get(&team) {
        println!("Imaging Team {}...", team);
        image(iface, team, key.clone()).await?;
        println!("Radio imaged successfully!");
      } else {
        println!("No key for team {}", team)
      }
    },
    None => interactive::run_interactive(iface, all_keys)?,
  }

  Ok(())
}

mod interactive {
  use cursive::{views::{Dialog, TextView, LinearLayout, EditView, PaddedView}, view::{Resizable, Identifiable, Margins}, theme::{ColorStyle, Color, BaseColor}};
  use jms_util::{net, WPAKeys};

  use crate::{imager::image};

  #[derive(Clone)]
  struct Data {
    keys: WPAKeys,
    team: u16
  }
  
  pub fn run_interactive(iface: net::LinkMetadata, keys: WPAKeys) -> anyhow::Result<()> {
    let mut siv = cursive::crossterm();
    
    siv.set_user_data(Data {
      team: 0,
      keys
    });
    
    siv.add_layer(Dialog::new()
      .title("Radio Imaging Tool")
      .padding(Margins::lrtb(1, 1, 1, 0))
      .content(
        LinearLayout::vertical()
          .child(
            LinearLayout::horizontal()
              .child(TextView::new("Team Number: "))
              .child(EditView::new().on_edit(|s, number, _| {
                // s.with_user_data(|dat: &mut Data| {
                let t = {
                  let dat = s.user_data::<Data>().unwrap();
                  dat.team = number.parse().unwrap_or(0);
                  (dat.team, dat.keys.contains_key(&dat.team))
                };

                let mut tv = s.find_name::<TextView>("msg").unwrap();
                tv.set_content(match t {
                  (0, _) => format!("Not a valid team number!"),
                  (t, false) => format!("No key found for Team {}", t),
                  (t, _) => format!("Ready to Image Team {}", t)
                });

                tv.set_style(ColorStyle::front(match t {
                  (0, _) => Color::Light(BaseColor::Red),
                  (_, false) => Color::Light(BaseColor::Magenta),
                  _ => Color::Light(BaseColor::Green)
                }));
              }).fixed_width(20).with_name("team"))
          )
          .child(
            PaddedView::lrtb(
              1, 0, 1, 1, 
              TextView::new(" ").with_name("msg")
            )
          )
      )
      .button("Image My Radio!", move |s| {
        // s.with_user_data(|dat: &mut Data| {
        let cb = s.cb_sink().clone();

        let dat = {
          s.user_data::<Data>().unwrap().clone()
        };

        if let Some(key) = dat.keys.get(&dat.team) {
          let team = dat.team;
          let key = key.clone();
          let i = iface.clone();

          {
            let mut tv = s.find_name::<TextView>("msg").unwrap();
            tv.set_content("Imaging Radio...");
            tv.set_style(ColorStyle::front(Color::Light(BaseColor::Magenta)));
          }
          {
            let mut but = s.find_name::<Dialog>("dialog").unwrap();
            but.buttons_mut().for_each(|b| b.disable());
          }

          let ah = tokio::runtime::Handle::current();
          // let (tx, rx) = channel::bounded(1);
          ah.spawn(async move {
            let result = image(i, team, key).await;
            // let _ = tx.send(result);
            cb.send(Box::new(move |s| {
              let mut tv = s.find_name::<TextView>("msg").unwrap();
              match result {
                Ok(()) => {
                  tv.set_content("Radio Imaged Successfully!");
                  tv.set_style(ColorStyle::front(Color::Light(BaseColor::Green)));
                }
                Err(err) => {
                  tv.set_content(format!("Imaging Failed. Try again. {}", err));
                  tv.set_style(ColorStyle::front(Color::Light(BaseColor::Red)));
                },
              }
              let mut but = s.find_name::<Dialog>("dialog").unwrap();
              but.buttons_mut().for_each(|b| b.enable());
            }))
          });
        }
      }).with_name("dialog")
    );

    siv.run();

    Ok(())
  }
}
