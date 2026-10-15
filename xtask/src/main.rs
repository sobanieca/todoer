//! Development tasks, run through the cargo aliases in `.cargo/config.toml`:
//!
//! - `cargo verify`: formatting, lints and tests
//! - `cargo update-snapshots`: accepts new CLI test output

use std::process::{Command, ExitCode};

fn cargo(args: &[&str], env: &[(&str, &str)]) -> bool {
    println!("> cargo {}", args.join(" "));
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    Command::new(cargo)
        .args(args)
        .envs(env.iter().copied())
        .status()
        .is_ok_and(|status| status.success())
}

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    let ok = match task.as_str() {
        "verify" => {
            cargo(&["fmt", "--all", "--check"], &[])
                && cargo(
                    &[
                        "clippy",
                        "--workspace",
                        "--all-targets",
                        "--",
                        "-D",
                        "warnings",
                    ],
                    &[],
                )
                && cargo(&["test", "--workspace"], &[])
        }
        "update-snapshots" => cargo(&["test", "--test", "cli"], &[("UPDATE_SNAPSHOTS", "1")]),
        _ => {
            eprintln!("Usage: cargo xtask <verify|update-snapshots>");
            false
        }
    };
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
