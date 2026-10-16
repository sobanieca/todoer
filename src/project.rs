//! Finds the checkout of the current folder, holds the lock, and makes the commits of todoer on
//! the current branch.

use crate::git;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

pub const DATA_DIR: &str = ".todoer";

pub struct Project {
    pub root: PathBuf,
    pub cwd: PathBuf,
}

pub struct CommitPlan {
    pub commit: bool,
    pub others: Vec<String>,
}

impl Project {
    pub fn find() -> Result<Project, String> {
        let cwd = std::env::current_dir()
            .map_err(|error| format!("cannot read the current folder: {error}"))?;
        Project::find_from(&cwd)
    }

    pub fn find_from(cwd: &Path) -> Result<Project, String> {
        let root = git::toplevel(cwd).ok_or_else(|| {
            "this folder is not in a git repository - run todoer in a project".to_string()
        })?;
        Ok(Project {
            root,
            cwd: cwd.to_path_buf(),
        })
    }

    pub fn data_dir(&self) -> PathBuf {
        self.root.join(DATA_DIR)
    }

    pub fn me(&self) -> Result<String, String> {
        git::user_email(&self.root)
    }

    pub fn relative(&self, argument: &str) -> Result<String, String> {
        relative_to(&self.root, &self.cwd, argument)
    }

    pub fn branch_name(&self) -> Result<String, String> {
        Ok(git::branch(&self.root)?.unwrap_or_else(|| "a detached HEAD".to_string()))
    }

    pub fn lock(&self) -> Result<Lock, String> {
        Lock::take(&git::common_dir(&self.root)?.join("todoer.lock"))
    }

    pub fn plan_commit(&self, no_commit: bool, own: &[String]) -> Result<CommitPlan, String> {
        let status = git::check(
            &self.root,
            &["status", "--porcelain=v1", "--untracked-files=no"],
        )?;
        let others = status
            .stdout
            .lines()
            .filter_map(|line| line.get(3..))
            .map(|path| path.rsplit(" -> ").next().unwrap_or(path).to_string())
            .filter(|path| !own.iter().any(|mine| is_inside(path, mine)))
            .collect::<Vec<_>>();
        Ok(CommitPlan {
            commit: !no_commit && others.is_empty(),
            others,
        })
    }

    pub fn commit(&self, lock: &Lock, message: &str, paths: &[String]) -> Result<(), String> {
        self.commit_paths(lock, &["-m", message], paths)
    }

    pub fn amend(&self, lock: &Lock, paths: &[String]) -> Result<(), String> {
        self.commit_paths(lock, &["--amend", "--no-edit"], paths)
    }

    fn commit_paths(&self, _lock: &Lock, options: &[&str], paths: &[String]) -> Result<(), String> {
        let mut present = Vec::new();
        for path in paths {
            if self.root.join(path).exists() || git::is_tracked(&self.root, path)? {
                present.push(path.as_str());
            }
        }
        if present.is_empty() {
            return Ok(());
        }
        let mut add = vec!["add", "-A", "-f", "--"];
        add.extend(&present);
        retry_on_index_lock(|| git::run(&self.root, &add))?;
        let mut commit = vec!["commit", "-q", "--no-verify"];
        commit.extend(options);
        commit.push("--");
        commit.extend(&present);
        retry_on_index_lock(|| git::run(&self.root, &commit))
    }

    pub fn commit_to_branch(
        &self,
        _lock: &Lock,
        branch: &str,
        message: &str,
        folder: &str,
        files: &[(String, String)],
    ) -> Result<(), String> {
        let root = &self.root;
        let reference = format!("refs/heads/{branch}");
        let old = git::text(root, &["rev-parse", "--verify", &reference])?;
        let index = git::git_path(root, "todoer-branch-index")?;
        let result = (|| {
            git::check_with_index(root, &["read-tree", &old], &index)?;
            git::check_with_index(
                root,
                &[
                    "rm",
                    "--cached",
                    "-r",
                    "-q",
                    "-f",
                    "--ignore-unmatch",
                    "--",
                    folder,
                ],
                &index,
            )?;
            for (path, text) in files {
                let blob = git::run_input(root, &["hash-object", "-w", "--stdin"], Some(text))?;
                if !blob.ok {
                    return Err(format!("git hash-object failed: {}", blob.message()));
                }
                let entry = format!("100644,{},{path}", blob.stdout.trim());
                git::check_with_index(
                    root,
                    &["update-index", "--add", "--cacheinfo", &entry],
                    &index,
                )?;
            }
            let tree = git::check_with_index(root, &["write-tree"], &index)?.stdout;
            let commit = git::run_input(
                root,
                &["commit-tree", tree.trim(), "-p", &old, "-F", "-"],
                Some(message),
            )?;
            if !commit.ok {
                return Err(format!("git commit-tree failed: {}", commit.message()));
            }
            git::check(
                root,
                &["update-ref", &reference, commit.stdout.trim(), &old],
            )?;
            Ok(())
        })();
        let _ = std::fs::remove_file(&index);
        result
    }
}

fn is_inside(path: &str, own: &str) -> bool {
    let own = own.trim_end_matches('/');
    path == own || path.starts_with(&format!("{own}/"))
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
    fn finds_the_paths_inside_a_folder() {
        assert!(is_inside(".todoer/prs/a/pr.md", ".todoer/prs/a"));
        assert!(is_inside(".todoer/prs/a", ".todoer/prs/a/"));
        assert!(!is_inside(".todoer/prs/ab/pr.md", ".todoer/prs/a"));
    }
}
