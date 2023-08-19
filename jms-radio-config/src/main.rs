mod imager;

use clap::Parser;
use imager::image;
use jms_util::{net::{self, LinkMetadata}, WPAKeys};
use tokio::{net::TcpStream, io::AsyncReadExt};

use crate::imager::ImagingProps;

#[derive(Parser, Debug)]
struct Args {
  /// The interface to run the imager on. If not provided, will prompt.
  #[clap(short, long, value_parser)]
  iface: Option<String>,
  /// In field mode, the CSV file of keys (or JMS if left blank). For home use, the WPA key
  #[clap(short, long, value_parser)]
  key: Option<String>,
  /// The team to image. If not provided, will run in interactive mode
  #[clap(value_parser)]
  team: Option<u16>,
  /// The SSID. By default, this is the team number. Ignored if interactive (no team specified)
  #[clap(short, long, value_parser)]
  ssid: Option<String>,
  /// Set field mode
  #[clap(short, long, action)]
  field: bool,
  /// Set basic field mode (no advanced networking)
  #[clap(short, long, action)]
  basic: bool
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
  
  let args = Args::parse();
  let iface = args.iface.clone().and_then(|i| valid_ifaces.iter().find(|iface| iface.name == i));
  let iface = iface.cloned().unwrap_or_else(|| {
    inquire::Select::<LinkMetadata>::new("Select Interface", valid_ifaces).prompt().unwrap()
  });
  
  if args.field {
    field_mode(&args, iface).await?;
  } else if args.basic {
    basic_mode(&args, iface).await?;
  } else {
    home_mode(&args, iface).await?;
  }

  Ok(())
}

async fn field_mode(args: &Args, iface: LinkMetadata) -> anyhow::Result<()> {
  let mut all_keys = WPAKeys::new();

  match args.key.clone() {
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
      let mut stream = TcpStream::connect("10.0.100.5:7171").await?;
      let len = stream.read_u32().await? as usize;

      let mut buf = vec![0; len];
      stream.read_exact(&mut buf).await?;

      let new_keys: WPAKeys = serde_json::from_slice(&buf)?;
      for (team, key) in new_keys.into_iter() {
        all_keys.insert(team, key);
      }
    },
  };

  match args.team {
    Some(team) => {
      if let Some(key) = all_keys.get(&team) {
        println!("Imaging Team {}...", team);
        image(iface, ImagingProps {
          team, 
          ssid: args.ssid.clone().unwrap_or(format!("{}", team)),
          key: key.clone(),
          home: false
        }).await?;
        println!("Radio imaged successfully!");
      } else {
        println!("No key for team {}", team)
      }
    },
    None => interactive::run_interactive(iface, interactive::InteractiveProps::Field(all_keys))?,
  }

  Ok(())
}

async fn basic_mode(args: &Args, iface: LinkMetadata) -> anyhow::Result<()> {
  match (args.ssid.clone(), args.team) {
    (Some(ssid), Some(team)) => {
      println!("Imaging Basic Mode: Team {}, Event SSID {}", team, ssid);
      image(iface, ImagingProps {
        team,
        ssid,
        key: args.key.clone().unwrap_or("".to_owned()),
        home: false
      }).await?;
      println!("Radio imaged successfully!");
    },
    _ => interactive::run_interactive(iface, interactive::InteractiveProps::Basic)?
  }
  Ok(())
}

async fn home_mode(args: &Args, iface: LinkMetadata) -> anyhow::Result<()> {
  match args.team {
    Some(team) => {
      println!("Imaging Team {} (home use)...", team);
      image(iface, ImagingProps {
        team, 
        ssid: args.ssid.clone().unwrap_or(format!("{}", team)),
        key: args.key.clone().unwrap_or("".to_owned()),
        home: true
      }).await?;
      println!("Radio imaged successfully!");
    },
    None => interactive::run_interactive(iface, interactive::InteractiveProps::Home(args.key.clone()))?,
  };
  Ok(())
}

mod interactive {
  use cursive::{views::{Dialog, TextView, LinearLayout, EditView, PaddedView}, view::{Resizable, Identifiable, Margins}, theme::{ColorStyle, Color, BaseColor}};
  use jms_util::{net, WPAKeys};

  use crate::imager::{image, ImagingProps};

  #[derive(Clone)]
  struct Data {
    props: InteractiveProps,
    ssid: String,
    team: u16,
    home_key: String
  }

  #[derive(Clone)]
  pub enum InteractiveProps {
    Field(WPAKeys),
    Basic,
    Home(Option<String>)
  }
  
  pub fn run_interactive(iface: net::LinkMetadata, props: InteractiveProps) -> anyhow::Result<()> {
    let mut siv = cursive::crossterm();
    
    siv.set_user_data(Data {
      team: 0,
      ssid: "".to_owned(),
      props: props.clone(),
      home_key: match &props {
        InteractiveProps::Field(_) => "".to_owned(),
        InteractiveProps::Basic => "".to_owned(),
        InteractiveProps::Home(key) => key.clone().unwrap_or("".to_owned()),
      }
    });
    
    let mut layout = LinearLayout::vertical();
    
    // Team Number Config
    let tn = LinearLayout::horizontal()
      .child(TextView::new("Team Number: "))
      .child(EditView::new().on_edit(|s, number, _| {
        // s.with_user_data(|dat: &mut Data| {
        let t = {
          let dat = s.user_data::<Data>().unwrap();
          dat.team = number.parse().unwrap_or(0);
          match &dat.props {
            InteractiveProps::Field(keys) => ( dat.team, keys.contains_key(&dat.team) ),
            InteractiveProps::Basic => ( dat.team, dat.team > 0 ),
            InteractiveProps::Home { .. } => ( dat.team, dat.team > 0 ),
          }
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
      }).fixed_width(20).with_name("team"));
    layout.add_child(tn);

    // Specific children per mode
    match props {
      InteractiveProps::Home(_) => {
        let ssid_layout = LinearLayout::horizontal()
          .child(TextView::new("SSID: "))
          .child(EditView::new().on_edit(|s, ssid, _| {
            let dat = s.user_data::<Data>().unwrap();
            dat.ssid = ssid.to_owned();
          }).fixed_width(27).with_name("ssid"));
        layout.add_child(ssid_layout);

        let key_layout = LinearLayout::horizontal()
          .child(TextView::new("WPA Key: "))
          .child(EditView::new().on_edit(|s, key, _| {
            let dat = s.user_data::<Data>().unwrap();
            dat.home_key = key.to_owned();
          }).fixed_width(24));
        layout.add_child(key_layout);
      },
      InteractiveProps::Basic => {
        let ssid_layout = LinearLayout::horizontal()
          .child(TextView::new("Event SSID: "))
          .child(EditView::new().on_edit(|s, ssid, _| {
            let dat = s.user_data::<Data>().unwrap();
            dat.ssid = ssid.to_owned();
          }).fixed_width(27).with_name("ssid"));
        layout.add_child(ssid_layout);

        let key_layout = LinearLayout::horizontal()
          .child(TextView::new("Event WPA Key: "))
          .child(EditView::new().on_edit(|s, key, _| {
            let dat = s.user_data::<Data>().unwrap();
            dat.home_key = key.to_owned();
          }).fixed_width(24));
        layout.add_child(key_layout);
      },
      InteractiveProps::Field(_) => ()
    }
    
    // Status message
    layout.add_child(PaddedView::lrtb(
      1, 0, 1, 1, 
      TextView::new(" ").with_name("msg")
    ));

    siv.add_layer(Dialog::new()
      .title("Radio Imaging Tool")
      .padding(Margins::lrtb(1, 1, 1, 0))
      .content(layout)
      .button("Image My Radio!", move |s| {
        // s.with_user_data(|dat: &mut Data| {
        let cb = s.cb_sink().clone();

        let dat = {
          s.user_data::<Data>().unwrap().clone()
        };

        let key = match &dat.props {
          _ if dat.team == 0 => None,   // Don't image radios without a team number set
          InteractiveProps::Field(keys) => keys.get(&dat.team).map(Clone::clone),
          InteractiveProps::Home(_) | InteractiveProps::Basic => Some(dat.home_key.clone()),
        };

        let ssid = match &dat.props {
          InteractiveProps::Field(_) => format!("{}", dat.team),
          InteractiveProps::Home(_) | InteractiveProps::Basic => if dat.ssid.len() == 0 { format!("{}", dat.team) } else { dat.ssid },
        };

        let home = matches!(dat.props, InteractiveProps::Home(_));

        if let Some(key) = key {
          let team = dat.team;
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
            let result = image(i, ImagingProps {
              team, ssid, key, home
            }).await;
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
