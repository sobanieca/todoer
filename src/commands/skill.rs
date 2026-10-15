//! `todoer skill`: prints the agent workflow, or writes it as a skill.

use crate::args::{Args, Flag, switch};
use crate::project::{Project, write_file};

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
    let project = Project::find()?;
    for (path, _) in TARGETS {
        let file = project.current.join(path);
        if file.exists() {
            return Err(format!(
                "{} exists - remove it to write the skill again",
                file.display()
            ));
        }
    }
    for (path, read_by) in TARGETS {
        write_file(&project.current.join(path), SKILL)?;
        println!("wrote {path}  ({read_by})");
    }
    Ok(())
}
