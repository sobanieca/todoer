//! The snapshot test harness: runs `todoer` through `sh -c` in a temporary git repository with its
//! own home folder, and compares the output with the files in `tests/snapshots`.
//!
//! `UPDATE_SNAPSHOTS=1` writes the snapshots again.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn bin_dir() -> PathBuf {
    Path::new(env!("CARGO_BIN_EXE_todoer"))
        .parent()
        .expect("binary folder")
        .to_path_buf()
}

pub struct Session {
    root: PathBuf,
    pub repo: PathBuf,
    transcript: String,
}

impl Session {
    pub fn new(name: &str) -> Session {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("todoer-test-{name}-{}-{id}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("home")).expect("make home");
        std::fs::create_dir_all(root.join("repo")).expect("make repo");
        let root = root.canonicalize().expect("temporary folder");
        std::fs::write(
            root.join("home/.gitconfig"),
            "[user]\n\temail = me@example.com\n\tname = Me\n[init]\n\tdefaultBranch = main\n",
        )
        .expect("write git config");
        let session = Session {
            repo: root.join("repo"),
            root,
            transcript: String::new(),
        };
        session.quiet("git init -q && git commit -q --allow-empty -m init");
        session
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub fn write(&self, relative: &str, text: &str) {
        let path = self.repo.join(relative);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("make folder");
        std::fs::write(path, text).expect("write file");
    }

    pub fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.repo.join(relative)).unwrap_or_default()
    }

    pub fn exec(&self, command: &str, cwd: &Path) -> (i32, String, String) {
        let path = format!(
            "{}:{}",
            bin_dir().display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(cwd)
            .env("PATH", path)
            .env("HOME", self.root.join("home"))
            .env("XDG_CONFIG_HOME", self.root.join("config"))
            .env("XDG_CACHE_HOME", self.root.join("cache"))
            .env("GIT_CONFIG_GLOBAL", self.root.join("home/.gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
            .env("TZ", "UTC")
            .env("TODOER_NO_UPDATE_CHECK", "1")
            .env_remove("NO_COLOR")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .stdin(Stdio::null())
            .output()
            .expect("run sh");
        (
            output.status.code().unwrap_or(-1),
            self.normalize(&String::from_utf8_lossy(&output.stdout)),
            self.normalize(&String::from_utf8_lossy(&output.stderr)),
        )
    }

    pub fn quiet(&self, command: &str) {
        let (code, stdout, stderr) = self.exec(command, &self.repo);
        assert_eq!(code, 0, "{command} failed:\n{stdout}{stderr}");
    }

    pub fn run_in(&mut self, command: &str, cwd: &Path) -> String {
        let (code, stdout, stderr) = self.exec(command, cwd);
        let location = match cwd.strip_prefix(&self.root) {
            Ok(relative) if relative != Path::new("repo") => {
                format!(" [cwd: {}]", relative.display())
            }
            _ => String::new(),
        };
        let command = command.replace(&self.root.display().to_string(), "<tmp>");
        let mut entry = format!("$ {command}{location}\n");
        if code != 0 {
            entry.push_str(&format!("exit code: {code}\n"));
        }
        entry.push_str(&stdout);
        if !stderr.is_empty() {
            entry.push_str("--- stderr\n");
            entry.push_str(&stderr);
        }
        entry.push('\n');
        self.transcript.push_str(&entry);
        stdout
    }

    pub fn run(&mut self, command: &str) -> String {
        let repo = self.repo.clone();
        self.run_in(command, &repo)
    }

    pub fn note(&mut self, title: &str, content: &str) {
        self.transcript
            .push_str(&format!("--- {title}\n{}\n", self.normalize(content)));
    }

    pub fn note_file(&mut self, relative: &str) {
        let text = self.read(relative);
        self.note(relative, &text);
    }

    pub fn assert_snapshot(&self, name: &str) {
        assert_snapshot(name, &self.transcript);
    }

    fn normalize(&self, text: &str) -> String {
        text.replace(&self.root.display().to_string(), "<tmp>")
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub fn assert_snapshot(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(format!("{name}.snap"));
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().expect("snapshot folder")).expect("snapshot folder");
        std::fs::write(&path, actual).expect("write snapshot");
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing snapshot {}, run cargo update-snapshots\n\n{actual}",
            path.display()
        )
    });
    if expected != actual {
        panic!(
            "snapshot {name} does not match (run cargo update-snapshots to accept)\n--- expected\n{expected}\n--- actual\n{actual}"
        );
    }
}
