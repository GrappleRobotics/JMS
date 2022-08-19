mod imager;

use std::{cell::RefCell, rc::Rc, collections::HashMap, sync::{Arc, Mutex}};

use crossbeam::channel;
use cursive::{views::{Dialog, SelectView, TextView, Panel, LinearLayout, EditView, Checkbox, PaddedView}, view::{Scrollable, Resizable, Identifiable, Margins}, Cursive, theme::{Effect, ColorStyle, Color, BaseColor}, With};
use imager::image;
use jms_util::net::{self, LinkMetadata};

use tokio::runtime::Handle as AsyncHandle;

struct Data {
  team: u16,
  teams: HashMap<u16, String>
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut siv = cursive::crossterm();

  let ifaces = net::get_all_ifaces(&net::handle().unwrap()).await?;
  let teams = load_keys().await?;
  
  siv.set_user_data(Data {
    team: 0,
    teams
  });

  
  prompt_iface(&mut siv,  ifaces, |s, iface| {
    s.add_layer(Dialog::new()
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
                 (dat.team, dat.teams.contains_key(&dat.team))
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
          s.user_data::<Data>().unwrap()
        };

        if let Some(key) = dat.teams.get(&dat.team) {
          let team = dat.team;
          let key = key.clone();
          let i = iface.clone();

          {
            let mut tv = s.find_name::<TextView>("msg").unwrap();
            tv.set_content("Imaging Radio...");
            tv.set_style(ColorStyle::front(Color::Light(BaseColor::Magenta)));

            let mut but = s.find_name::<Dialog>("dialog").unwrap();
            but.buttons_mut().for_each(|b| b.disable());
          }

          let ah = AsyncHandle::current();
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

              let mut team = s.find_name::<EditView>("team").unwrap();
              team.set_content("");
            }))
          });
        }
      }).with_name("dialog")
    )
  })?;

  siv.run();

  Ok(())
}

fn prompt_iface<F>(s: &mut Cursive, ifaces: Vec<LinkMetadata>, cb: F) -> anyhow::Result<()>
where
  F: Fn(&mut Cursive, LinkMetadata) + 'static
{
  let mut select_view = SelectView::<LinkMetadata>::new().autojump();
  // select_view.add_all_str(ifaces.iter().map(|x| format!("{}", x)));
  select_view.add_all(ifaces.into_iter().map(|x| ( format!("{}", x), x )));

  select_view.set_on_submit(move |s, iface| {
    s.pop_layer();
    cb(s, iface.clone())
  });

  s.add_layer(
    Dialog::around(
      select_view.scrollable()
    ).title("Select Interface")
  );

  Ok(())
}

async fn load_keys() -> anyhow::Result<HashMap<u16, String>> {
  let mut map: HashMap<u16, String> = HashMap::new();
  map.insert(4788, "abcd".to_owned());
  return Ok(map)
}