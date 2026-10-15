//! Finds the main checkout from any worktree, reads `todoer.json`, and makes the commits of todoer
//! on the main branch.

use crate::git;
use crate::json::{self, Value};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

pub const DATA_DIR: &str = ".todoer";
pub const CONFIG_FILE: &str = "todoer.json";

#[derive(Debug, Default)]
pub struct Config {
    pub main_branch: Option<String>,
}

pub fn parse_config(text: &str) -> Result<Config, String> {
    let value =
        json::parse(text).map_err(|error| format!("{CONFIG_FILE} is not valid JSON: {error}"))?;
    let Value::Object(entries) = value else {
        return Err(format!("{CONFIG_FILE} must hold an object"));
    };
    let mut config = Config::default();
    for (key, entry) in entries {
        let text = match &entry {
            Value::String(text) if !text.is_empty() => Some(text.clone()),
            _ => None,
        };
        match key.as_str() {
            "mainBranch" => {
                config.main_branch = Some(text.ok_or_else(|| {
                    format!("invalid {CONFIG_FILE} - \"mainBranch\" must be a branch name")
                })?);
            }
            _ => return Err(format!("{CONFIG_FILE} has an unknown key \"{key}\"")),
        }
    }
    Ok(config)
}

pub struct Project {
    pub root: PathBuf,
    pub current: PathBuf,
    pub cwd: PathBuf,
    pub main_branch: String,
}

impl Project {
    pub fn find() -> Result<Project, String> {
        let cwd = std::env::current_dir()
            .map_err(|error| format!("cannot read the current folder: {error}"))?;
        Project::find_from(&cwd)
    }

    pub fn find_from(cwd: &Path) -> Result<Project, String> {
        let current = git::toplevel(cwd).ok_or_else(|| {
            "this folder is not in a git repository - run todoer in a project".to_string()
        })?;
        let worktrees = git::worktrees(cwd)?;
        let main = worktrees
            .first()
            .filter(|worktree| !worktree.bare)
            .ok_or_else(|| "todoer needs a repository with a main worktree".to_string())?;
        let root = main.path.clone();
        let config = match std::fs::read_to_string(root.join(CONFIG_FILE)) {
            Ok(text) => parse_config(&text)?,
            Err(_) => Config::default(),
        };
        let main_branch = match &config.main_branch {
            Some(branch) => branch.clone(),
            None => detect_main_branch(&root, main.branch.as_deref())?,
        };
        Ok(Project {
            root,
            current,
            cwd: cwd.to_path_buf(),
            main_branch,
        })
    }

    pub fn data_dir(&self) -> PathBuf {
        self.root.join(DATA_DIR)
    }

    pub fn me(&self) -> Result<String, String> {
        git::user_email(&self.current)
    }

    pub fn relative(&self, argument: &str) -> Result<String, String> {
        relative_to(&self.current, &self.cwd, argument)
    }

    pub fn require_main_checkout_on_main(&self) -> Result<(), String> {
        let branch = git::branch(&self.root)?;
        if branch.as_deref() == Some(self.main_branch.as_str()) {
            return Ok(());
        }
        Err(format!(
            "the main checkout {} is on {} - check out {} there first. todoer commits only on {}",
            self.root.display(),
            branch.as_deref().unwrap_or("no branch"),
            self.main_branch,
            self.main_branch
        ))
    }

    pub fn lock(&self) -> Result<Lock, String> {
        Lock::take(&git::common_dir(&self.root)?.join("todoer.lock"))
    }

    pub fn commit_on_main(
        &self,
        _lock: &Lock,
        message: &str,
        paths: &[String],
    ) -> Result<(), String> {
        self.require_main_checkout_on_main()?;
        let present: Vec<&str> = paths
            .iter()
            .map(String::as_str)
            .filter(|path| self.root.join(path).exists())
            .collect();
        if !present.is_empty() {
            let mut add = vec!["add", "-A", "-f", "--"];
            add.extend(present);
            retry_on_index_lock(|| git::run(&self.root, &add))?;
        }
        let mut commit = vec!["commit", "-q", "--no-verify", "-m", message, "--"];
        commit.extend(paths.iter().map(String::as_str));
        retry_on_index_lock(|| git::run(&self.root, &commit))
    }
}

fn retry_on_index_lock(
    mut action: impl FnMut() -> Result<git::Output, String>,
) -> Result<(), String> {
    let start = Instant::now();
    loop {
        let output = action()?;
        if output.ok {
            return Ok(());
        }
        if output.stderr.contains("index.lock") && start.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }
        return Err(format!("git commit failed: {}", output.message()));
    }
}

fn detect_main_branch(root: &Path, checked_out: Option<&str>) -> Result<String, String> {
    let remote = git::run(
        root,
        &[
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    )?;
    if remote.ok
        && let Some(name) = remote.stdout.trim().strip_prefix("origin/")
    {
        return Ok(name.to_string());
    }
    for name in ["main", "master"] {
        if git::branch_exists(root, name)? {
            return Ok(name.to_string());
        }
    }
    Ok(checked_out.unwrap_or("main").to_string())
}

pub fn relative_to(toplevel: &Path, cwd: &Path, argument: &str) -> Result<String, String> {
    let joined = if Path::new(argument).is_absolute() {
        PathBuf::from(argument)
    } else {
        cwd.join(argument)
    };
    let normal = normalize(&joined);
    let top = normalize(toplevel);
    let relative = normal.strip_prefix(&top).map_err(|_| {
        format!(
            "{argument} is outside the repository {}",
            toplevel.display()
        )
    })?;
    let text = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    if text.is_empty() {
        return Err(format!("{argument} is the repository root, not a file"));
    }
    Ok(text)
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn write_file(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("cannot make {}: {error}", parent.display()))?;
    }
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temporary = path.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
    std::fs::write(&temporary, text)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
    std::fs::rename(&temporary, path)
        .map_err(|error| format!("cannot write {}: {error}", path.display()))
}

pub fn read_file(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map(|text| text.replace("\r\n", "\n"))
        .map_err(|error| format!("cannot read {}: {error}", path.display()))
}

pub struct Lock {
    path: PathBuf,
}

impl Lock {
    fn take(path: &Path) -> Result<Lock, String> {
        let start = Instant::now();
        loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
            {
                Ok(_) => {
                    return Ok(Lock {
                        path: path.to_path_buf(),
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let stale = std::fs::metadata(path)
                        .and_then(|meta| meta.modified())
                        .ok()
                        .and_then(|time| SystemTime::now().duration_since(time).ok())
                        .is_some_and(|age| age > Duration::from_secs(300));
                    if stale {
                        let _ = std::fs::remove_file(path);
                        continue;
                    }
                    if start.elapsed() > Duration::from_secs(30) {
                        return Err(format!(
                            "another todoer command holds {} - wait for it, or remove the file if no todoer runs",
                            path.display()
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(error) => return Err(format!("cannot make {}: {error}", path.display())),
            }
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makes_paths_relative_to_the_repository() {
        let top = Path::new("/repo");
        assert_eq!(
            relative_to(top, Path::new("/repo/todo"), "a.md").unwrap(),
            "todo/a.md"
        );
        assert_eq!(
            relative_to(top, Path::new("/repo/todo"), "../b/c.md").unwrap(),
            "b/c.md"
        );
        assert_eq!(
            relative_to(top, Path::new("/repo"), "./x/./y.md").unwrap(),
            "x/y.md"
        );
        assert!(relative_to(top, Path::new("/repo"), "../out.md").is_err());
        assert!(relative_to(top, Path::new("/repo"), ".").is_err());
    }

    #[test]
    fn reads_the_config() {
        let config = parse_config(r#"{"mainBranch": "trunk"}"#).unwrap();
        assert_eq!(config.main_branch.as_deref(), Some("trunk"));
        assert!(
            parse_config(r#"{"port": 1}"#)
                .unwrap_err()
                .contains("unknown key \"port\"")
        );
        assert!(parse_config(r#"{"mainBranch": ""}"#).is_err());
        assert!(parse_config("[]").is_err());
    }
}
