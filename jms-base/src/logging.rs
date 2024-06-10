use std::{io::Write, sync::Mutex};
use log::Level;
use termcolor::{StandardStream, WriteColor, ColorSpec, Color};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct LogRecord {
  pub id: String,
  pub timestamp_utc: f64,
  pub level: String,
  pub target: String,
  pub message: String,
  pub module: Option<String>,
  pub file: Option<String>,
  pub line: Option<u32>
}

pub async fn auto_flush() {
  let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
  loop {
    interval.tick().await;
    log::logger().flush()
  }
}

pub struct FlushGuard { }

impl Drop for FlushGuard {
  fn drop(&mut self) {
    log::logger().flush()
  }
}

pub struct JMSLogger { }

impl JMSLogger {
  pub async fn init() -> anyhow::Result<FlushGuard> {
    let me = Self {};

    log::set_boxed_logger(Box::new(me)).map(|()| log::set_max_level(log::LevelFilter::Info))?;

    tokio::task::spawn(auto_flush());

    Ok(FlushGuard { })
  }
}

impl log::Log for JMSLogger {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    metadata.level() <= Level::Info
  }

  fn log(&self, record: &log::Record) {
    let meta = record.metadata();
    if self.enabled(meta) {
      let timestamp = chrono::Utc::now();

      let log_record = LogRecord {
        id: Uuid::new_v4().to_string(),
        timestamp_utc: timestamp.timestamp() as f64 + timestamp.timestamp_subsec_nanos() as f64 / 10e9f64,
        level: meta.level().to_string(),
        target: meta.target().to_owned(),
        message: format!("{}", record.args()),
        module: record.module_path().map(|x| x.to_owned()),
        file: record.file().map(|x| x.to_owned()),
        line: record.line().clone(),
      };

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

      let mut stdout = StandardStream::stdout(termcolor::ColorChoice::Always);
      stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Rgb(100, 100, 100)))).ok();
      write!(&mut stdout, "{} ", chrono::Local::now().format("%Y-%m-%d %H:%M:%S.%3f %z")).ok();
      stdout.set_color(&level_spec).ok();
      write!(&mut stdout, "{} ", log_record.level.to_uppercase()).ok();
      if let Some(module) = &log_record.module {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_bold(true)).ok();
        write!(&mut stdout, "{} ", &module).ok();
      }
      stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(100, 100, 100)))).ok();
      write!(&mut stdout, "> ").ok();
      stdout.set_color(&message_spec).ok();
      write!(&mut stdout, "{}", log_record.message).ok();
      if meta.level() <= Level::Warn {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(150, 150, 150)))).ok();
        write!(&mut stdout, " [at {}:{}]", log_record.file.clone().unwrap_or("<unknown>".to_owned()), log_record.line.map(|x| format!("{}", x)).unwrap_or("<unknown>".to_owned())).ok();
      }
      stdout.reset().ok();
      writeln!(&mut stdout).ok();

      let flush = meta.level() <= Level::Warn;

      if flush {
        self.flush();
      }
    }
  }

  fn flush(&self) { }
}