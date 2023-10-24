use std::{self, io::Error, fs::File};

// TODO: add file logging instruments, for a concrete logging system
//       that can be referenced for internal errors.

const LOG_FILE: &'static str = "term_log.txt";

use crate::core::settings::{CONFIG_PATH_DIR_ENVVAR, CONFIG_DIR_NAME};
use std::fs::OpenOptions;

/// get the folder where the log file should be, this doesnt 
/// actually assure that the log file exists.
pub fn find_log_location() -> Option<String> {
    if let Ok(dir) = std::env::var(CONFIG_PATH_DIR_ENVVAR) {
        Some(
            // Either %AppData%/Roaming/.termrs
            // Or /usr/home/.termrs
            format!("{dir}/{CONFIG_DIR_NAME}")
        )
    }
    else {
        None
    }
}

pub fn build_file(base_dir: &String) -> Result<File, Error> {
    let path = format!("{base_dir}/{LOG_FILE}");
    OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
}

#[macro_export]
macro_rules! log {
    ($fmt:literal, $($arg:tt)*) => {{
        use std::io::Write;
        if let Some(path) = find_log_location() {
            let data = format!($fmt, $($arg)*);
            if let Ok(mut f) = build_file(&path) {
                let _ = writeln!(f, "{}", data);
            }
        }
    }};
}

pub(crate) use log;