use std::{
  cell::RefCell,
  env, fmt,
  sync::atomic::{AtomicUsize, Ordering},
};

use chrono::Local;
use env_logger::{
  fmt::{Color, Style, StyledValue},
  Builder, Target,
};
use log::Level;
use regex::Regex;

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

// Breadcrumb support for logging with a trace of where things come from.

thread_local!(static BREADCRUMB: RefCell<Vec<String>> = RefCell::new(Vec::new()));

// Consider using context! instead.
#[allow(dead_code)]
pub fn push(name: &str) {
  BREADCRUMB.with(|bc| {
    bc.borrow_mut().push(String::from(name));
  });
}

// Consider using context! instead.
#[allow(dead_code)]
pub fn pop() {
  BREADCRUMB.with(|bc| {
    bc.borrow_mut().pop();
  });
}

// Used to load breadcrumbs across threads -> call breadcrumb() from the originating thread, and then load it in the thread.

#[allow(dead_code)]
pub fn store() -> Vec<String> {
  BREADCRUMB.with(|bc| bc.borrow().clone())
}

#[allow(dead_code)]
pub fn load(breadcrumb: &Vec<String>) {
  BREADCRUMB.with(|bc| bc.borrow_mut().clone_from(breadcrumb))
}

// TT munch to allow context stacking
#[macro_export(local_inner_macros)]
macro_rules! context {
  ($f:expr) => (
    $f
  );
  ($head:expr, $($further:tt)+) => {{
    $crate::logging::push($head);
    scopeguard::defer!($crate::logging::pop());
    context!($($further)+)
  }};
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

// Silencing
#[derive(Clone, PartialEq)]
pub struct SilenceCriteria {
  target: Option<String>,
  level: Option<Level>,
}

thread_local!(static SILENCERS: RefCell<Vec<SilenceCriteria>> = RefCell::new(Vec::new()));

#[allow(dead_code)]
pub fn silence(target: Option<&str>, level: Option<Level>) -> SilenceCriteria {
  let crit = SilenceCriteria {
    target: target.map(|s| String::from(s)),
    level,
  };
  SILENCERS.with(|s| {
    s.borrow_mut().push(crit.clone());
  });
  crit
}

#[allow(dead_code)]
pub fn unsilence(criteria: SilenceCriteria) {
  SILENCERS.with(|s| s.borrow_mut().retain(|x| *x != criteria));
}

// silenced!(target: Some("jms"), level: Some(Level::Info), { /* ...  */ })
// will silence all logs emitted by jms at Info or lower.
#[macro_export(local_inner_macros)]
macro_rules! silenced {
  (target: $t:expr, level: $l:expr, $f:expr) => {{
    let _sc = $crate::logging::silence($t, $l);
    scopeguard::defer!($crate::logging::unsilence(_sc));
    $f;
  }};
  (target: $t:expr, $f:expr) => {
    silenced!(target: $t, level: None, $f);
  };
  (level: $t:expr, $f:expr) => {
    silenced!(target: None, level: $t, $f);
  };
}

const COLOR_GRAY_DARK: Color = Color::Rgb(100, 100, 100);
const COLOR_GRAY: Color = Color::Rgb(150, 150, 150);

// Adapted from pretty_env_logger, with some custom sauce (better timestamps, error line/file refs, breadcrumb)
fn builder() -> Builder {
  let mut builder = Builder::new();

  builder.format(|f, record| {
    use std::io::Write;

    if is_silenced(record.target(), record.level()) {
      return Ok(());
    }

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
    let breadcrumb = style.set_color(COLOR_GRAY).value(render_breadcrumb());

    let mut style = f.style();
    let message = message_colored_level(&mut style, record.level()).value(record.args());

    let mut style = f.style();
    let lineno = render_record_line(&mut style, record.file(), record.line());

    let mut style = f.style();
    let splitter = style.set_color(COLOR_GRAY_DARK).set_bold(true).value(">");

    if record.level() <= Level::Error {
      writeln!(
        f,
        " {} {} {}{} {} {}: {}",
        time, level, target, breadcrumb, splitter, lineno, message
      )
    } else {
      writeln!(
        f,
        " {} {} {}{} {} {}",
        time, level, target, breadcrumb, splitter, message
      )
    }
  });

  builder
}

fn is_silenced(target: &str, level: Level) -> bool {
  SILENCERS.with(|s| {
    s.borrow().iter().any(|el| {
      let mut matched = [true, true];
      if let Some(t) = el.target.as_ref() {
        matched[0] = Regex::new(t).unwrap().is_match(target); // This will panic without a valid regex! Beware!
      }
      if let Some(l) = el.level {
        matched[1] = level >= l;
      }
      matched.iter().all(|x| *x)
    })
  })
}

fn render_breadcrumb() -> String {
  let joined = BREADCRUMB.with(|bc| bc.borrow().join("::"));
  if joined.is_empty() {
    joined
  } else {
    format!("[{}]", joined)
  }
}

fn render_record_line<'a>(
  style: &'a mut Style,
  file: Option<&str>,
  num: Option<u32>,
) -> StyledValue<'a, String> {
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
