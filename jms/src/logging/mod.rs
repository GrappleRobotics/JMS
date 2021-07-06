use std::{
  env, fmt,
  sync::atomic::{AtomicUsize, Ordering},
};

use chrono::Local;
use env_logger::{
  fmt::{Color, Style, StyledValue},
  Builder, Target,
};
use log::Level;

pub fn configure(debug_mode: bool) {
  let mut default_level = log::LevelFilter::Info;
  if debug_mode {
    default_level = log::LevelFilter::Debug;
  }

  let env_filters = env::var("JMS_LOG").unwrap_or_default();

  builder()
    .filter_level(default_level)
    .parse_filters(&env_filters)
    .target(Target::Stdout)
    .init();
}

// Error wrapping
#[macro_export(local_inner_macros)]
macro_rules! log_expect {
  ($result:expr, $($arg:tt)+) => {{
    $result.unwrap_or_else(|e| {
      log::error!($($arg)+, e);
      std::panic!($($arg)+, e);
    })
  }};
  (debug $result:expr) => {{
    log_expect!($result, "Unexpected FATAL error: {:?}")
  }};
  ($result:expr) => {{
    log_expect!($result, "Unexpected FATAL error: {}")
  }}
}

const COLOR_GRAY_DARK: Color = Color::Rgb(100, 100, 100);
const COLOR_GRAY: Color = Color::Rgb(150, 150, 150);

// Adapted from pretty_env_logger, with some custom sauce (better timestamps, error line/file refs, breadcrumb)
fn builder() -> Builder {
  let mut builder = Builder::new();

  builder.format(|f, record| {
    use std::io::Write;

    let target = record.target();

    let max_width = max_target_width(target);

    let mut style = f.style();
    let level = colored_level(&mut style, record.level());

    let mut style = f.style();
    let target = style.set_bold(true).value(Padded {
      value: target,
      width: max_width,
    });

    let mut style = f.style();
    let time = style
      .set_color(COLOR_GRAY_DARK)
      .value(Local::now().format("%Y-%m-%d %H:%M:%S.%3f %z"));

    let mut style = f.style();
    let message = message_colored_level(&mut style, record.level()).value(record.args());

    let mut style = f.style();
    let lineno = render_record_line(&mut style, record.file(), record.line());

    let mut style = f.style();
    let splitter = style.set_color(COLOR_GRAY_DARK).set_bold(true).value(">");

    if record.level() <= Level::Error {
      writeln!(
        f,
        " {} {} {} {} {}: {}",
        time, level, target, splitter, lineno, message
      )
    } else {
      writeln!(
        f,
        " {} {} {} {} {}",
        time, level, target, splitter, message
      )
    }
  });

  builder
}

fn render_record_line<'a>(style: &'a mut Style, file: Option<&str>, num: Option<u32>) -> StyledValue<'a, String> {
  let file = file.unwrap_or("<unknown>");
  let ln = match num {
    Some(n) => n.to_string(),
    None => String::from("<unknown>"),
  };
  return style.set_bold(true).value(format!("[at {}:{}]", file, ln));
}

// from pretty_env_logger
struct Padded<T> {
  value: T,
  width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{: <width$}", self.value, width = self.width)
  }
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
  match level {
    Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
    Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
    Level::Info => style.set_color(Color::Green).value(" INFO"),
    Level::Warn => style.set_color(Color::Yellow).value(" WARN"),
    Level::Error => style.set_color(Color::Red).value("ERROR"),
  }
}

fn message_colored_level(style: &mut Style, level: Level) -> &mut Style {
  match level {
    Level::Trace => style.set_color(COLOR_GRAY_DARK),
    Level::Debug => style.set_color(COLOR_GRAY),
    Level::Info => style.set_color(Color::White),
    Level::Warn => style.set_color(Color::Yellow),
    Level::Error => style.set_color(Color::Red).set_bold(true),
  }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);
static ABSOLUTE_MAX: usize = 20;

fn max_target_width(target: &str) -> usize {
  let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
  if max_width < target.len() && target.len() < ABSOLUTE_MAX {
    MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
    target.len()
  } else {
    max_width
  }
}
