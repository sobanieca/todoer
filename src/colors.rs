//! ANSI colors. They are on only when the output is a terminal and `NO_COLOR` is not set.

use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);

pub fn init(terminal: bool) {
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
    ENABLED.store(terminal && !no_color, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

fn paint(text: &str, open: &str, close: &str) -> String {
    if !enabled() || text.is_empty() {
        return text.to_string();
    }
    format!("\x1b[{open}m{text}\x1b[{close}m")
}

pub fn bold(text: &str) -> String {
    paint(text, "1", "22")
}

pub fn red(text: &str) -> String {
    paint(text, "31", "39")
}

pub fn green(text: &str) -> String {
    paint(text, "32", "39")
}

pub fn yellow(text: &str) -> String {
    paint(text, "33", "39")
}

pub fn magenta(text: &str) -> String {
    paint(text, "35", "39")
}

pub fn cyan(text: &str) -> String {
    paint(text, "36", "39")
}

pub fn gray(text: &str) -> String {
    paint(text, "90", "39")
}

pub fn strip(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

pub fn width(text: &str) -> usize {
    strip(text).chars().count()
}

pub fn pad(text: &str, size: usize) -> String {
    let fill = size.saturating_sub(width(text));
    format!("{text}{}", " ".repeat(fill))
}

pub fn pad_left(text: &str, size: usize) -> String {
    let fill = size.saturating_sub(width(text));
    format!("{}{text}", " ".repeat(fill))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_and_pads() {
        assert_eq!(strip("\x1b[31mred\x1b[39m"), "red");
        assert_eq!(pad("\x1b[31mab\x1b[39m", 4), "\x1b[31mab\x1b[39m  ");
        assert_eq!(pad_left("ab", 4), "  ab");
    }
}
