//! Runs git as a child process.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub struct Output {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    pub fn message(&self) -> String {
        let raw = if self.stderr.trim().is_empty() {
            &self.stdout
        } else {
            &self.stderr
        };
        let lines: Vec<&str> = raw
            .lines()
            .filter(|line| !line.starts_with("hint:"))
            .collect();
        let text = lines.join("\n");
        let text = text.trim();
        if text.is_empty() {
            "no output".to_string()
        } else {
            text.to_string()
        }
    }
}

fn command(cwd: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    command
        .args(["-c", "core.quotePath=false"])
        .args(args)
        .current_dir(cwd)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE");
    command
}

fn spawn_error(error: std::io::Error) -> String {
    if error.kind() == std::io::ErrorKind::NotFound {
        "git is not installed - todoer needs git on the PATH".to_string()
    } else {
        format!("cannot run git: {error}")
    }
}

pub fn run(cwd: &Path, args: &[&str]) -> Result<Output, String> {
    run_input(cwd, args, None)
}

pub fn run_input(cwd: &Path, args: &[&str], input: Option<&str>) -> Result<Output, String> {
    let mut child = command(cwd, args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(spawn_error)?;
    if let Some(input) = input
        && let Some(mut stdin) = child.stdin.take()
    {
        let _ = stdin.write_all(input.as_bytes());
    }
    let output = child.wait_with_output().map_err(spawn_error)?;
    Ok(Output {
        ok: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

pub fn check(cwd: &Path, args: &[&str]) -> Result<Output, String> {
    let output = run(cwd, args)?;
    if !output.ok {
        return Err(format!(
            "git {} failed: {}",
            args.first().unwrap_or(&""),
            output.message()
        ));
    }
    Ok(output)
}

pub fn text(cwd: &Path, args: &[&str]) -> Result<String, String> {
    Ok(check(cwd, args)?.stdout.trim().to_string())
}

pub fn toplevel(cwd: &Path) -> Option<PathBuf> {
    let output = run(cwd, &["rev-parse", "--show-toplevel"]).ok()?;
    output.ok.then(|| PathBuf::from(output.stdout.trim()))
}

pub fn branch(cwd: &Path) -> Result<Option<String>, String> {
    let output = run(cwd, &["symbolic-ref", "--quiet", "--short", "HEAD"])?;
    Ok(output.ok.then(|| output.stdout.trim().to_string()))
}

pub fn branch_exists(cwd: &Path, name: &str) -> Result<bool, String> {
    Ok(run(
        cwd,
        &[
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{name}"),
        ],
    )?
    .ok)
}

pub fn common_dir(cwd: &Path) -> Result<PathBuf, String> {
    text(
        cwd,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .map(PathBuf::from)
}

pub fn user_email(cwd: &Path) -> Result<String, String> {
    let output = run(cwd, &["config", "user.email"])?;
    let email = output.stdout.trim().to_string();
    if !output.ok || email.is_empty() {
        return Err("git has no user.email - run git config user.email <you@example.com>".into());
    }
    Ok(email)
}

pub struct Worktree {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub bare: bool,
}

pub fn parse_worktrees(porcelain: &str) -> Vec<Worktree> {
    let mut worktrees: Vec<Worktree> = Vec::new();
    for line in porcelain.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            worktrees.push(Worktree {
                path: PathBuf::from(path),
                branch: None,
                bare: false,
            });
        } else if let Some(current) = worktrees.last_mut() {
            if let Some(branch) = line.strip_prefix("branch ") {
                current.branch = Some(branch.trim_start_matches("refs/heads/").to_string());
            } else if line == "bare" {
                current.bare = true;
            }
        }
    }
    worktrees
}

pub fn worktrees(cwd: &Path) -> Result<Vec<Worktree>, String> {
    Ok(parse_worktrees(
        &check(cwd, &["worktree", "list", "--porcelain"])?.stdout,
    ))
}

pub fn show(cwd: &Path, revision: &str, path: &str) -> Result<Option<String>, String> {
    let output = run(cwd, &["show", &format!("{revision}:{path}")])?;
    Ok(output.ok.then_some(output.stdout))
}

pub fn is_tracked(cwd: &Path, path: &str) -> Result<bool, String> {
    Ok(run(cwd, &["ls-files", "--error-unmatch", "--", path])?.ok)
}

pub fn has_changes(cwd: &Path, path: &str) -> Result<bool, String> {
    let output = check(cwd, &["status", "--porcelain", "--", path])?;
    Ok(!output.stdout.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_worktree_list() {
        let list = parse_worktrees(
            "worktree /repo\nHEAD abc\nbranch refs/heads/main\n\nworktree /repo/wt\nHEAD def\ndetached\n",
        );
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].branch.as_deref(), Some("main"));
        assert_eq!(list[1].branch, None);
        assert_eq!(list[1].path, PathBuf::from("/repo/wt"));
    }
}
