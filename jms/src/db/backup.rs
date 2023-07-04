use std::{path::PathBuf, time::Duration, fs::{File, self}, io::{Write, Read}};

use chrono::Local;
use humantime::format_duration;
use tokio::time::interval;
use walkdir::WalkDir;
use zip::write::FileOptions;

use crate::{config::Interactive, models, arena::Arena};

use super::TableType;

pub struct DBBackup {
  pub arena: Arena,
  pub settings: DBBackupSettings
}

impl DBBackup {
  pub fn new(arena: Arena, settings: DBBackupSettings) -> Self {
    Self { arena, settings }
  }

  pub async fn run(self) -> anyhow::Result<()> {
    let mut interval = interval(self.settings.frequency);

    let db = super::database();
    let mut watch_matches = models::Match::table(db)?.watch_all();

    loop {
      let mut update = false;

      tokio::select! {
        m = watch_matches.get() => {
          match m? {
            // Only backup when a match is finished
            super::WatchEvent::Insert(v) if v.data.played  => {
              update = true;
            },
            super::WatchEvent::Remove(_) => {
              update = true;
            },
            _ => ()
          };
        },
        _ = interval.tick() => {
          info!("Backup Tick");
          if self.arena.can_backup().await {
            update = true;
          }
        }
      }

      if update {
        info!("Taking a backup...");
        match self.zip() {
          Ok(()) => info!("Backup complete!"),
          Err(e) => error!("Backup error - {}", e)
        }
      }
    }
  }

  // From zip-rs examples
  fn zip(&self) -> anyhow::Result<()> {
    let now = Local::now().format("%Y%m%d-%H-%M-%S");
    let mut path = self.settings.path.clone();
    path.push(format!("jms-backup-{}.zip", now));
    
    let cfg = super::database().config();
    let walk = WalkDir::new((*cfg).get_path());

    fs::create_dir_all(path.parent().unwrap())?;
    let out = File::create(&path)?;
  
    let mut writer = zip::ZipWriter::new(out);

    for entry in walk.into_iter() {
      let de = entry?;
      let path = de.path();
      let name = path.strip_prefix((*cfg).get_path())?;

      if path.is_file() {
        writer.start_file(name.to_string_lossy(), FileOptions::default())?;
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        writer.write_all(&*buf)?;
        buf.clear();
      } else if path.as_os_str().len() > 0 {
        writer.add_directory(name.to_string_lossy(), FileOptions::default())?;
      }
    }

    Ok(())
  }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DBBackupSettings {
  pub path: PathBuf,
  pub frequency: Duration
}

#[async_trait::async_trait]
impl Interactive for DBBackupSettings {
  async fn interactive() -> anyhow::Result<Self> {
    let path = inquire::Text::new("Database backup path:").with_default("JMS_BACKUP").prompt()?;
    let frequency = inquire::CustomType::<Duration> {
      message: "How often should database backups occur?",
      formatter: &|val| format_duration(val).to_string(),
      error_message: "Please enter a valid duration (e.g. 5 minutes)".into(),
      help_message: "This should be in a duration format, e.g. 5 minutes.".into(),
      parser: &|i| match i.parse::<humantime::Duration>() {
        Ok(v) => Ok(v.into()),
        Err(_) => Err(()),
      },
      default: Some((Duration::from_secs(5*60), &|val| format_duration(val).to_string())),
      placeholder: None,
      render_config: inquire::ui::RenderConfig::default()
    }.prompt()?;
    
    Ok(Self {
      path: path.into(), 
      frequency
    })
  }
}