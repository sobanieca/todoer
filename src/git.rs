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

pub fn run_with_index(cwd: &Path, args: &[&str], index: &Path) -> Result<Output, String> {
    let output = command(cwd, args)
        .env("GIT_INDEX_FILE", index)
        .stdin(Stdio::null())
        .output()
        .map_err(spawn_error)?;
    Ok(Output {
        ok: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

pub fn check_with_index(cwd: &Path, args: &[&str], index: &Path) -> Result<Output, String> {
    let output = run_with_index(cwd, args, index)?;
    if !output.ok {
        return Err(format!(
            "git {} failed: {}",
            args.first().unwrap_or(&""),
            output.message()
        ));
    }
    Ok(output)
}

pub fn git_path(cwd: &Path, name: &str) -> Result<PathBuf, String> {
    text(
        cwd,
        &["rev-parse", "--path-format=absolute", "--git-path", name],
    )
    .map(PathBuf::from)
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

pub fn check_input(cwd: &Path, args: &[&str], input: &str) -> Result<Output, String> {
    let output = run_input(cwd, args, Some(input))?;
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

pub fn show(cwd: &Path, revision: &str, path: &str) -> Result<Option<String>, String> {
    let output = run(cwd, &["show", &format!("{revision}:{path}")])?;
    Ok(output.ok.then_some(output.stdout))
}

pub fn local_branch_exists(cwd: &Path, branch: &str) -> Result<bool, String> {
    Ok(run(
        cwd,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ],
    )?
    .ok)
}

pub fn checkout_of_branch(cwd: &Path, branch: &str) -> Result<Option<PathBuf>, String> {
    let list = check(cwd, &["worktree", "list", "--porcelain"])?.stdout;
    let wanted = format!("branch refs/heads/{branch}");
    for block in list.split("\n\n") {
        let mut path = None;
        let mut matches = false;
        for line in block.lines() {
            if let Some(rest) = line.strip_prefix("worktree ") {
                path = Some(PathBuf::from(rest));
            } else if line == wanted {
                matches = true;
            }
        }
        if let Some(path) = path.filter(|path| matches && path.is_dir()) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

pub fn is_tracked(cwd: &Path, path: &str) -> Result<bool, String> {
    Ok(run(cwd, &["ls-files", "--error-unmatch", "--", path])?.ok)
}
