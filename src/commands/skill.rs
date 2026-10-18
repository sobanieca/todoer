//! `todoer skill`: prints the agent workflow, or writes it as a skill.

use crate::args::{Args, Flag, switch};
use crate::git;
use crate::project::write_file;
use std::path::PathBuf;

pub const SKILL: &str = include_str!("../templates/SKILL.md");
const TARGETS: [(&str, &str); 2] = [
    (".agents/skills/todoer/SKILL.md", "Codex, Cursor"),
    (".claude/skills/todoer/SKILL.md", "Claude Code, Cursor"),
];

pub const FLAGS: &[Flag] = &[switch("init", None)];

pub fn workflow() -> &'static str {
    SKILL
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map(|(_, body)| body.trim_start())
        .unwrap_or(SKILL)
}

pub fn run(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 0, "todoer skill [--init]")?;
    if !args.has("init") {
        print!("{}", workflow());
        return Ok(());
    }
    let cwd = std::env::current_dir()
        .map_err(|error| format!("cannot read the current folder: {error}"))?;
    let (root, global) = match git::toplevel(&cwd) {
        Some(root) => (root, false),
        None => (home_dir()?, true),
    };
    for (path, _) in TARGETS {
        let file = root.join(path);
        if file.exists() {
            return Err(format!(
                "{} exists - remove it to write the skill again",
                file.display()
            ));
        }
    }
    if global {
        println!(
            "not in a git repository - writing the skill to {} for all projects",
            root.display()
        );
    }
    for (path, read_by) in TARGETS {
        write_file(&root.join(path), SKILL)?;
        println!("wrote {path}  ({read_by})");
    }
    Ok(())
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .filter(|home| !home.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| {
            "this folder is not in a git repository and HOME is not set - set HOME".to_string()
        })
}
