//! Messages on stderr. The result of a command goes to stdout.

use crate::colors;

pub fn warn(message: &str) {
    eprintln!("{} {message}", colors::yellow("warning:"));
}

pub fn error(message: &str) {
    eprintln!("{} {message}", colors::red("error:"));
}

pub fn info(message: &str) {
    eprintln!("{message}");
}
