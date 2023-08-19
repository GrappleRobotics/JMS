use std::io::Write;
use log::Level;
use termcolor::{StandardStream, WriteColor, ColorSpec, Color};
use tokio::runtime::Handle;
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct LogRecord {
  pub id: String,
  pub timestamp_utc: i64,
  pub level: String,
  pub target: String,
  pub message: String,
  pub module: Option<String>,
  pub file: Option<String>,
  pub line: Option<u32>
}

pub struct JMSLogger {
  meili_client: meilisearch_sdk::Client,
}

impl JMSLogger {
  pub fn init() -> anyhow::Result<()> {
    let me = Self {
      meili_client: meilisearch_sdk::Client::new("http://localhost:7700", None::<String>)
    };

    log::set_boxed_logger(Box::new(me)).map(|()| log::set_max_level(log::LevelFilter::Info))?;
    Ok(())
  }
}

impl log::Log for JMSLogger {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    metadata.level() <= Level::Info
  }

  fn log(&self, record: &log::Record) {
    let meta = record.metadata();
    if self.enabled(meta) {
      let index = self.meili_client.index("logs");
      let timestamp = chrono::Utc::now();

      let log_record = LogRecord {
        id: Uuid::new_v4().to_string(),
        timestamp_utc: timestamp.timestamp(),
        level: meta.level().to_string(),
        target: meta.target().to_owned(),
        message: format!("{}", record.args()),
        module: record.module_path().map(|x| x.to_owned()),
        file: record.file().map(|x| x.to_owned()),
        line: record.line().clone(),
      };

      let handle = Handle::current();
      let _ = handle.enter();
      match futures::executor::block_on(index.add_documents(&[log_record.clone()], Some("id"))) {
        Ok(_) => (),
        Err(e) => eprintln!("Logging Failure (Meilisearch): {}", e)
      }

      let mut level_spec = ColorSpec::new();
      let mut message_spec = ColorSpec::new();

      match meta.level() {
        Level::Error => {
          level_spec.set_fg(Some(Color::Red)).set_bold(true);
          message_spec.set_fg(Some(Color::Red)).set_bold(true);
        },
        Level::Warn => {
          level_spec.set_fg(Some(Color::Yellow));
          message_spec.set_fg(Some(Color::Yellow));
        },
        Level::Info => {
          level_spec.set_fg(Some(Color::Green));
          message_spec.set_fg(Some(Color::White));
        },
        Level::Debug => {
          level_spec.set_fg(Some(Color::Blue));
          message_spec.set_fg(Some(Color::Rgb(150, 150, 150)));
        },
        Level::Trace => {
          level_spec.set_fg(Some(Color::Magenta));
          message_spec.set_fg(Some(Color::Rgb(100, 100, 100)));
        },
      }

      let mut stdout = StandardStream::stdout(termcolor::ColorChoice::Auto);
      stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Rgb(100, 100, 100)))).ok();
      write!(&mut stdout, "{} ", chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f %z")).ok();
      stdout.set_color(&level_spec).ok();
      write!(&mut stdout, "{} ", log_record.level.to_uppercase()).ok();
      if let Some(module) = log_record.module {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_bold(true)).ok();
        write!(&mut stdout, "{} ", &module).ok();
      }
      stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(100, 100, 100)))).ok();
      write!(&mut stdout, "> ").ok();
      stdout.set_color(&message_spec).ok();
      write!(&mut stdout, "{}", log_record.message).ok();
      if meta.level() <= Level::Warn {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(150, 150, 150)))).ok();
        write!(&mut stdout, " [at {}:{}]", log_record.file.unwrap_or("<unknown>".to_owned()), log_record.line.map(|x| format!("{}", x)).unwrap_or("<unknown>".to_owned())).ok();
      }
      stdout.reset().ok();
      writeln!(&mut stdout).ok();
    }
  }

  fn flush(&self) { }
}