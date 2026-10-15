//! Pull requests: a folder `.todoer/prs/<slug>/` on the main branch with `pr.md`, the patch, the
//! diff and the review of the latest submit.

use crate::diff;
use crate::frontmatter::Document;
use crate::git;
use crate::pr_view;
use crate::project::{DATA_DIR, Lock, Project, read_file, write_file};
use crate::review::{self, Comment, Review, Target, Tree};
use crate::todos::{read_todo, slug};
use std::path::{Path, PathBuf};

const EXCLUDE_DATA: [&str; 3] = ["--", ".", ":(exclude).todoer"];
pub const REVIEW: &str = "review.md";
pub const PATCH: &str = "pr.patch";
pub const DIFF: &str = "pr.diff";
const UPDATE_HEADING: &str = "## Update";

#[derive(Debug, Clone)]
pub struct Pr {
    pub slug: String,
    pub dir: PathBuf,
    pub title: String,
    pub author: String,
    pub branch: String,
    pub base: String,
    pub todos: Vec<String>,
    pub doc: Document,
}

pub fn prs_dir(project: &Project) -> PathBuf {
    project.data_dir().join("prs")
}

fn valid_slug(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

impl Pr {
    fn load(dir: &Path, slug: &str) -> Result<Pr, String> {
        let doc = Document::parse(&read_file(&dir.join("pr.md"))?);
        Ok(Pr {
            slug: slug.to_string(),
            dir: dir.to_path_buf(),
            title: doc.text("title").unwrap_or_else(|| slug.to_string()),
            author: doc.text("author").unwrap_or_default(),
            branch: doc.text("branch").unwrap_or_default(),
            base: doc.text("base").unwrap_or_default(),
            todos: doc.list("todos"),
            doc,
        })
    }

    fn save(&mut self) -> Result<(), String> {
        self.doc.set_text("title", &self.title);
        self.doc.set("author", &self.author);
        self.doc.set("branch", &self.branch);
        self.doc.set("base", &self.base);
        self.doc.set_list("todos", &self.todos);
        write_file(&self.dir.join("pr.md"), &self.doc.render())
    }

    pub fn description(&self) -> &str {
        self.doc.body.trim()
    }

    pub fn rel_dir(&self) -> String {
        format!("{DATA_DIR}/prs/{}", self.slug)
    }

    pub fn rel(&self, name: &str) -> String {
        format!("{}/{name}", self.rel_dir())
    }

    pub fn diff_text(&self) -> String {
        std::fs::read_to_string(self.dir.join(DIFF)).unwrap_or_default()
    }

    pub fn files(&self) -> Vec<diff::File> {
        diff::parse(&self.diff_text())
    }

    pub fn review(&self) -> Review {
        match std::fs::read_to_string(self.dir.join(REVIEW)) {
            Ok(text) => Review::parse(&text),
            Err(_) => Review::empty(),
        }
    }

    pub fn committed_review(&self, project: &Project) -> Review {
        match git::show(&project.root, "HEAD", &self.rel(REVIEW)) {
            Ok(Some(text)) => Review::parse(&text),
            _ => Review::empty(),
        }
    }

    pub fn drafts(&self, project: &Project) -> Vec<Comment> {
        let committed = self.committed_review(project);
        self.review()
            .comments
            .into_iter()
            .filter(|comment| !committed.comments.iter().any(|old| old.id == comment.id))
            .collect()
    }

    pub fn verdict(&self, _project: &Project) -> String {
        self.review().verdict
    }

    pub fn commits(&self) -> usize {
        std::fs::read_to_string(self.dir.join(PATCH))
            .unwrap_or_default()
            .lines()
            .filter(|line| line.starts_with("From ") && line.ends_with("Mon Sep 17 00:00:00 2001"))
            .count()
    }
}

pub struct PrTree<'a> {
    project: &'a Project,
    pr: &'a Pr,
    files: Vec<diff::File>,
}

impl Pr {
    pub fn tree<'a>(&'a self, project: &'a Project) -> PrTree<'a> {
        PrTree {
            project,
            pr: self,
            files: self.files(),
        }
    }
}

impl Tree for PrTree<'_> {
    fn lines(&self, path: &str) -> Option<Vec<String>> {
        match self.files.iter().find(|file| file.matches(path)) {
            Some(file) if file.new_path.as_deref() == Some(path) => {
                pr_view::new_lines(self.project, self.pr, file)
            }
            Some(_) => None,
            None => pr_view::base_lines(self.project, self.pr, path),
        }
    }

    fn has_folder(&self, path: &str) -> bool {
        git::text(
            &self.project.root,
            &["cat-file", "-t", &format!("{}:{path}", self.pr.base)],
        )
        .is_ok_and(|kind| kind == "tree")
    }
}

pub fn list(project: &Project) -> Result<Vec<Pr>, String> {
    let dir = prs_dir(project);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join("pr.md").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
        .iter()
        .map(|name| Pr::load(&dir.join(name), name))
        .collect()
}

pub fn resolve(project: &Project, argument: &str) -> Result<Pr, String> {
    let name = argument
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(argument)
        .to_string();
    let dir = prs_dir(project).join(&name);
    if !valid_slug(&name) || !dir.join("pr.md").is_file() {
        return Err(format!(
            "there is no open pull request {name} - run todoer pr ls"
        ));
    }
    Pr::load(&dir, &name)
}

pub struct Submitted {
    pub pr: Pr,
    pub created: bool,
    pub commits: usize,
    pub files: usize,
    pub cleared: Option<Review>,
}

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

fn check_no_drafts(project: &Project, pr: &Pr) -> Result<(), String> {
    let drafts = pr.drafts(project);
    if drafts.is_empty() {
        return Ok(());
    }
    let ids: Vec<&str> = drafts.iter().map(|comment| comment.id.as_str()).collect();
    Err(format!(
        "a review of {} is in progress, with the draft comments {} - the reviewer must send it first with todoer pr request-changes",
        pr.slug,
        ids.join(", ")
    ))
}

pub fn submit(
    project: &Project,
    title: Option<&str>,
    message: Option<&str>,
    todo_paths: &[String],
    update: bool,
) -> Result<Submitted, String> {
    project.require_main_checkout_on_main()?;
    let cwd = &project.current;
    let branch = git::branch(cwd)?
        .ok_or("the current checkout has no branch - check out the branch of your work")?;
    if branch == project.main_branch {
        return Err(format!(
            "this checkout is on {} - commit your work on a branch of its own, in a worktree",
            project.main_branch
        ));
    }
    let mut status = vec!["status", "--porcelain"];
    status.extend(EXCLUDE_DATA);
    let dirty = git::text(cwd, &status)?;
    if !dirty.is_empty() {
        return Err(format!(
            "the worktree has changes that are not committed - commit them first:\n{dirty}"
        ));
    }
    let base = git::text(cwd, &["merge-base", "HEAD", &project.main_branch])?;
    let range = format!("{base}..HEAD");
    let commits = git::text(cwd, &["rev-list", "--count", &range])?
        .parse::<usize>()
        .unwrap_or(0);
    if commits == 0 {
        return Err(format!(
            "there is no commit on {branch} after {} - commit your work first",
            project.main_branch
        ));
    }
    let mut format_patch = vec!["format-patch", "--stdout", &range];
    format_patch.extend(EXCLUDE_DATA);
    let patch = git::check(cwd, &format_patch)?.stdout;
    let mut diff_args = vec!["diff", "-M", &base, "HEAD"];
    diff_args.extend(EXCLUDE_DATA);
    let diff_text = git::check(cwd, &diff_args)?.stdout;
    if diff_text.trim().is_empty() {
        return Err(format!(
            "the commits on {branch} change no file outside {DATA_DIR}/"
        ));
    }
    for path in todo_paths {
        read_todo(cwd, path)?;
    }
    let author = git::user_email(cwd)?;
    let lock = project.lock()?;
    let existing = list(project)?.into_iter().find(|pr| pr.branch == branch);
    let created = existing.is_none();
    let cleared = match &existing {
        Some(pr) => {
            check_no_drafts(project, pr)?;
            let review = pr.committed_review(project);
            if !review.comments.is_empty() && !update {
                return Err(format!(
                    "{} is already submitted, and its review has {} - do what the comments ask, then run todoer pr submit --update to replace the patch and remove the review",
                    pr.slug,
                    plural(review.comments.len(), "comment")
                ));
            }
            Some(review)
        }
        None => None,
    };
    let mut pr = match existing {
        Some(pr) => pr,
        None => {
            let first_subject = git::text(cwd, &["log", "--reverse", "--format=%s", &range])?
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            let title = title
                .map(str::to_string)
                .filter(|title| !title.trim().is_empty())
                .unwrap_or(first_subject);
            let base_slug = match slug(&title, 40) {
                name if name.is_empty() => slug(&branch, 40),
                name => name,
            };
            let base_slug = if base_slug.is_empty() {
                "pr".to_string()
            } else {
                base_slug
            };
            let mut name = base_slug.clone();
            let mut number = 2;
            while prs_dir(project).join(&name).exists() {
                name = format!("{base_slug}-{number}");
                number += 1;
            }
            let mut doc = Document::new();
            doc.body = String::new();
            Pr {
                slug: name.clone(),
                dir: prs_dir(project).join(&name),
                title,
                author: author.clone(),
                branch: branch.clone(),
                base: String::new(),
                todos: Vec::new(),
                doc,
            }
        }
    };
    if let Some(title) = title.filter(|title| !title.trim().is_empty()) {
        pr.title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    pr.base = base;
    for path in todo_paths {
        if !pr.todos.contains(path) {
            pr.todos.push(path.clone());
        }
    }
    let message = message.map(str::trim).filter(|text| !text.is_empty());
    let description = pr
        .description()
        .split(&format!("\n{UPDATE_HEADING}\n"))
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let (description, update) = match (&cleared, message) {
        (None, _) => (message.unwrap_or("").to_string(), String::new()),
        (Some(_), Some(message)) => (description, format!("{UPDATE_HEADING}\n\n{message}")),
        (Some(_), None) => (description, String::new()),
    };
    pr.doc.body = match (description.is_empty(), update.is_empty()) {
        (true, true) => String::new(),
        (false, true) => format!("\n{description}\n"),
        (true, false) => format!("\n{update}\n"),
        (false, false) => format!("\n{description}\n\n{update}\n"),
    };
    write_file(&pr.dir.join(PATCH), &patch)?;
    write_file(&pr.dir.join(DIFF), &diff_text)?;
    pr.save()?;
    let mut paths = vec![pr.rel("pr.md"), pr.rel(PATCH), pr.rel(DIFF)];
    if pr.dir.join(REVIEW).exists() {
        git::check(
            &project.root,
            &["rm", "-q", "--ignore-unmatch", "-f", "--", &pr.rel(REVIEW)],
        )?;
        let _ = std::fs::remove_file(pr.dir.join(REVIEW));
        paths.push(pr.rel(REVIEW));
    }
    let action = if created { "submit" } else { "update" };
    let commit_message = format!("todoer: {action} {}", pr.slug);
    let committed = project.commit_on_main(&lock, &commit_message, &paths);
    if let Err(error) = committed {
        if created {
            let _ = std::fs::remove_dir_all(&pr.dir);
        }
        return Err(error);
    }
    let files = diff::parse(&diff_text).len();
    Ok(Submitted {
        pr,
        created,
        commits,
        files,
        cleared,
    })
}

fn am_message(output: &git::Output) -> String {
    let lines: Vec<&str> = output
        .stdout
        .lines()
        .chain(output.stderr.lines())
        .filter(|line| {
            line.starts_with("CONFLICT")
                || line.starts_with("error:")
                || line.starts_with("Patch failed")
        })
        .collect();
    if lines.is_empty() {
        output.message()
    } else {
        lines.join("\n")
    }
}

pub struct Merged {
    pub commits: usize,
    pub done: Vec<String>,
    pub warnings: Vec<String>,
}

fn check_no_comments(project: &Project, pr: &Pr) -> Result<(), String> {
    check_no_drafts(project, pr)?;
    let comments = pr.review().comments;
    if comments.is_empty() {
        return Ok(());
    }
    let ids: Vec<&str> = comments.iter().map(|comment| comment.id.as_str()).collect();
    Err(format!(
        "{} has the unresolved comments {} - the author must do what they ask, then run todoer pr submit --update",
        pr.slug,
        ids.join(", ")
    ))
}

pub fn merge(project: &Project, pr: &Pr) -> Result<Merged, String> {
    let lock = project.lock()?;
    project.require_main_checkout_on_main()?;
    check_no_comments(project, pr)?;
    let root = &project.root;
    if !git::run(root, &["diff", "--cached", "--quiet"])?.ok {
        return Err(format!(
            "{} has staged changes - commit or unstage them first",
            root.display()
        ));
    }
    let patch_text = pr.dir.join(PATCH).to_string_lossy().into_owned();
    let applied = git::run(root, &["am", "-q", "--3way", &patch_text])?;
    if !applied.ok {
        let _ = git::run(root, &["am", "--abort"]);
        return Err(format!(
            "the patch does not apply on {} - {} did not change. Ask the author to rebase the branch and submit again:\n{}",
            project.main_branch,
            project.main_branch,
            am_message(&applied)
        ));
    }
    let commits = pr.commits();
    let mut warnings = Vec::new();
    let mut done = Vec::new();
    let mut paths = Vec::new();
    for path in &pr.todos {
        match read_todo(root, path) {
            Ok(mut todo) => {
                if todo.status != "done" {
                    todo.set_status("done");
                    todo.save(root)?;
                    paths.push(path.clone());
                }
                done.push(path.clone());
            }
            Err(error) => warnings.push(format!("todo {path}: {error}")),
        }
    }
    finish(
        project,
        &lock,
        pr,
        paths,
        &format!("todoer: merge {}", pr.slug),
    )?;
    Ok(Merged {
        commits,
        done,
        warnings,
    })
}

fn finish(
    project: &Project,
    lock: &Lock,
    pr: &Pr,
    mut paths: Vec<String>,
    message: &str,
) -> Result<(), String> {
    let dir = pr.rel_dir();
    git::check(
        &project.root,
        &["rm", "-r", "-q", "--ignore-unmatch", "--", &dir],
    )?;
    let _ = std::fs::remove_dir_all(&pr.dir);
    paths.push(dir);
    project.commit_on_main(lock, message, &paths)
}

pub fn close(project: &Project, pr: &Pr) -> Result<(), String> {
    let lock = project.lock()?;
    project.require_main_checkout_on_main()?;
    finish(
        project,
        &lock,
        pr,
        Vec::new(),
        &format!("todoer: close {}", pr.slug),
    )
}

pub fn add_comment(
    project: &Project,
    pr: &Pr,
    target: Option<&str>,
    text: &str,
) -> Result<Comment, String> {
    let target = match target {
        Some(text) => review::parse_target(text)?,
        None => Target::Pr,
    };
    let quote = review::check_target(&target, &pr.files(), &pr.tree(project))?;
    let text = review::clean_text(text)?;
    project.require_main_checkout_on_main()?;
    let author = project.me()?;
    let _lock = project.lock()?;
    let mut review = pr.review();
    let comment = Comment {
        id: review.next_id(),
        target,
        author,
        quote,
        text,
    };
    review.comments.push(comment.clone());
    write_file(&pr.dir.join(REVIEW), &review.render())?;
    Ok(comment)
}

pub fn set_verdict(
    project: &Project,
    pr: &Pr,
    verdict: &str,
    message: Option<&str>,
) -> Result<Review, String> {
    let reviewer = project.me()?;
    let lock = project.lock()?;
    project.require_main_checkout_on_main()?;
    let mut review = pr.review();
    review.verdict = verdict.to_string();
    review.reviewer = Some(reviewer);
    if let Some(message) = message.map(str::trim).filter(|text| !text.is_empty()) {
        review.summary = message.to_string();
    }
    write_file(&pr.dir.join(REVIEW), &review.render())?;
    let action = if verdict == "approved" {
        "approve"
    } else {
        "request changes on"
    };
    project.commit_on_main(
        &lock,
        &format!("todoer: {action} {}", pr.slug),
        &[pr.rel(REVIEW)],
    )?;
    Ok(review)
}
