//! The terminal: stdin and `$PAGER`.

use std::io::{IsTerminal, Read, Write};
use std::process::{Command, Stdio};

pub fn stdin_is_terminal() -> bool {
    std::io::stdin().is_terminal()
}

pub fn stdout_is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

pub fn read_stdin_if_piped() -> Result<String, String> {
    if stdin_is_terminal() {
        return Ok(String::new());
    }
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|error| format!("cannot read stdin: {error}"))?;
    Ok(text)
}

pub fn page(text: &str) {
    if !stdout_is_terminal() {
        print!("{text}");
        let _ = std::io::stdout().flush();
        return;
    }
    let pager = std::env::var("PAGER")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "less".to_string());
    let child = Command::new("sh")
        .arg("-c")
        .arg(&pager)
        .env(
            "LESS",
            std::env::var("LESS").unwrap_or_else(|_| "FRX".to_string()),
        )
        .stdin(Stdio::piped())
        .spawn();
    match child {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
        Err(_) => print!("{text}"),
    }
}
