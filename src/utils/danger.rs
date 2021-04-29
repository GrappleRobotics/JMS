use std::{env, fs, str::FromStr};

use crate::warn;
use lazy_static::lazy_static;
use lexical_bool::{LexicalBool, initialize_false_values, initialize_true_values};

lazy_static! {
  pub static ref IS_DANGER_ZONE: bool = eval_danger_zone();
}

const JMS_DANGER_FILE: &str = "/etc/jms-danger-zone";
const JMS_DANGER_ENV_VAR: &str = "JMS_DANGER_ENABLED";
const JMS_DANGER_STRING: &str = "I CONSENT TO JMS DESTROYING MY COMPUTER";

fn eval_danger_zone() -> bool {
  let mut danger = false;

  initialize_true_values(&["1", "true", "t", "yes", "y"]);
  initialize_false_values(&["0", "false", "f", "no", "n"]);

  // Attempt to parse file
  let file_content = fs::read_to_string(JMS_DANGER_FILE);
  match file_content {
    Ok(s) => {
      if s.trim() != JMS_DANGER_STRING {
        warn!("The content of {} is unexpected. To enable danger zone, the file should have content \"{}\" only.", JMS_DANGER_FILE, JMS_DANGER_STRING);
      } else {
        danger = true;
      }
    },
    _ => ()
  }

  // Parse env var
  match env::var(JMS_DANGER_ENV_VAR) {
    Ok(s) => {
      match LexicalBool::from_str(&s.to_lowercase()) {
        Ok(lb) => danger = *lb,
        Err(_) => warn!("Couldn't parse {}. Are you sure the value \"{}\" is correct?", JMS_DANGER_ENV_VAR, s)
      }
    },
    _ => ()
  }

  if danger {
    warn!("======!!!!======= DANGER ZONE ENABLED ======!!!!=======");
    warn!("| JMS is in production mode. JMS will override system |");
    warn!("|      configurations to setup the field network      |");
    warn!("|                                                     |");
    warn!("|  If this is not what you intend, stop JMS now and   |");
    warn!("|     delete the {} file and/or     |", JMS_DANGER_FILE);
    warn!("|              unset {}.              |", JMS_DANGER_ENV_VAR);
    warn!("================= DANGER ZONE ENABLED =================");
  } else {
    warn!("JMS is in safe mode. To allow JMS to modify system files, populate the {} file or {} env var.", JMS_DANGER_FILE, JMS_DANGER_ENV_VAR);
  }

  danger
}