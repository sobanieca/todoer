//! Pull requests: a folder `.todoer/prs/<name>/` with `pr.md`, the patch, the diff and the review
//! of the latest submit. A submit writes the folder in the working tree of the branch that the pull
//! request goes into. todoer never commits it. The patch carries every change of the work, also in
//! `.todoer/`, except the folder of the pull request itself. The todos that a pull request removes
//! are the todos that it does.

use crate::diff;
use crate::frontmatter::Document;
use crate::git;
use crate::pr_view;
use crate::project::{DATA_DIR, Lock, Project, read_file, write_file};
use crate::review::{self, Comment, Review, Target, Tree};
use crate::todos::{remove_todo, slug, todo_file, todo_path, valid_name};
use std::path::{Path, PathBuf};

pub const REVIEW: &str = "review.md";
pub const PATCH: &str = "pr.patch";
pub const DIFF: &str = "pr.diff";

#[derive(Debug, Clone)]
pub struct Pr {
    pub name: String,
    pub dir: PathBuf,
    pub author: String,
    pub base: String,
    pub todos: Vec<String>,
    pub doc: Document,
}

pub fn prs_dir(project: &Project) -> PathBuf {
    project.data_dir().join("prs")
}

fn pr_dir_path(name: &str) -> String {
    format!("{DATA_DIR}/prs/{name}")
}

/// The pathspec of a submit: all files, except the folder of the pull request.
fn without_pr(name: Option<&str>) -> Vec<String> {
    let mut pathspec = vec!["--".to_string(), ".".to_string()];
    if let Some(name) = name {
        pathspec.push(format!(":(exclude){}", pr_dir_path(name)));
    }
    pathspec
}

impl Pr {
    fn load(dir: &Path, name: &str) -> Result<Pr, String> {
        let doc = Document::parse(&read_file(&dir.join("pr.md"))?);
        let diff_text = std::fs::read_to_string(dir.join(DIFF)).unwrap_or_default();
        Ok(Pr {
            name: name.to_string(),
            dir: dir.to_path_buf(),
            author: doc.text("author").unwrap_or_default(),
            base: doc.text("base").unwrap_or_default(),
            todos: removed_todos(&diff::parse(&diff_text)),
            doc,
        })
    }

    fn render(&mut self) -> String {
        self.doc.set("author", &self.author);
        self.doc.set("base", &self.base);
        self.doc.render()
    }

    pub fn description(&self) -> &str {
        self.doc.body.trim()
    }

    pub fn rel_dir(&self) -> String {
        pr_dir_path(&self.name)
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

/// The names of the todo files that the diff removes.
fn removed_todos(files: &[diff::File]) -> Vec<String> {
    files
        .iter()
        .filter(|file| file.new_path.is_none())
        .filter_map(|file| file.old_path.as_deref())
        .filter_map(|path| path.strip_prefix(&format!("{DATA_DIR}/todos/")))
        .filter_map(|file| file.strip_suffix(".md"))
        .filter(|name| valid_name(name))
        .map(str::to_string)
        .collect()
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

/// The open pull requests that remove the todo.
pub fn of_todo<'a>(prs: &'a [Pr], name: &str) -> Vec<&'a Pr> {
    prs.iter()
        .filter(|pr| pr.todos.iter().any(|todo| todo == name))
        .collect()
}

/// `<name> (<n> comments)` for each pull request.
pub fn names_with_comments(prs: &[&Pr]) -> Vec<String> {
    prs.iter()
        .map(|pr| match pr.review().comments.len() {
            0 => pr.name.clone(),
            1 => format!("{} (1 comment)", pr.name),
            count => format!("{} ({count} comments)", pr.name),
        })
        .collect()
}

/// `pr: <name> (<n> comments), ...` for the pull requests of a todo.
pub fn note(prs: &[&Pr]) -> Option<String> {
    if prs.is_empty() {
        return None;
    }
    Some(format!("pr: {}", names_with_comments(prs).join(", ")))
}

pub fn resolve(project: &Project, argument: &str) -> Result<Pr, String> {
    let name = argument
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(argument)
        .to_string();
    let dir = prs_dir(project).join(&name);
    if !valid_name(&name) || !dir.join("pr.md").is_file() {
        return Err(format!(
            "there is no open pull request {name} - run todoer pr ls"
        ));
    }
    Pr::load(&dir, &name)
}

pub struct Submitted {
    pub pr: Pr,
    pub into: String,
    pub from: String,
    pub created: bool,
    pub kept_description: bool,
    pub commits: usize,
    pub files: usize,
    pub cleared: Option<Review>,
}

pub struct SubmitOptions<'a> {
    pub branch: Option<&'a str>,
    pub name: Option<&'a str>,
    pub message: Option<&'a str>,
    pub update: bool,
}

fn check_name(name: &str) -> Result<(), String> {
    if valid_name(name) {
        return Ok(());
    }
    match slug(name, 60) {
        good if good.is_empty() => Err(format!(
            "\"{name}\" is not a pull request name - use lowercase letters, digits and hyphens, with a letter or digit at each end"
        )),
        good => Err(format!(
            "\"{name}\" is not a pull request name - use {good}"
        )),
    }
}

/// The message of the squash in `pr.patch`, and the message that `pr apply --merge` proposes.
fn merge_message(name: &str) -> String {
    format!("todoer: merge {name}")
}

fn open_names(into: &str, names: &[String]) -> String {
    if names.is_empty() {
        format!("{into} has no open pull requests")
    } else {
        format!("The open pull requests on {into}: {}", names.join(", "))
    }
}

fn check_clean(project: &Project, pr: &Pr, branch: &str) -> Result<(), String> {
    let others = project.changes(&pr.rel_dir())?;
    if others.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{branch} has changes that are not committed: {} - commit them first",
        others.join(", ")
    ))
}

fn check_work_committed(project: &Project, from: &str, name: Option<&str>) -> Result<(), String> {
    let Some(checkout) = git::checkout_of_branch(&project.root, from)? else {
        return Ok(());
    };
    let ignored = name.map(pr_dir_path).unwrap_or_default();
    let others = Project::find_from(&checkout)?.changes(&ignored)?;
    if others.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{from} has changes that are not committed in {}: {} - commit them first",
        checkout.display(),
        others.join(", ")
    ))
}

pub fn submit(project: &Project, options: &SubmitOptions) -> Result<Submitted, String> {
    let root = &project.root;
    let into = git::branch(root)?.ok_or(
        "the current checkout has no branch - check out the branch that the pull request goes into",
    )?;
    let from = options
        .branch
        .filter(|from| !from.trim().is_empty())
        .ok_or("a submit needs the branch of the work - check out the branch that the pull request goes into, and run todoer pr submit --branch <the branch of the work>")?
        .to_string();
    if from == into {
        return Err(format!(
            "the pull request cannot come from {into}, the branch that is checked out - check out the branch that it goes into, and run todoer pr submit --branch {into} there"
        ));
    }
    if !git::local_branch_exists(root, &from)? {
        return Err(format!(
            "there is no local branch {from} - check the name in --branch"
        ));
    }
    if let Some(name) = options.name {
        check_name(name)?;
    }
    check_work_committed(project, &from, options.name)?;
    let pathspec = without_pr(options.name);
    let pathspec: Vec<&str> = pathspec.iter().map(String::as_str).collect();
    let _lock = project.lock()?;
    let open = list(project)?;
    let merge_base = git::run(root, &["merge-base", "HEAD", &from])?;
    if !merge_base.ok {
        return Err(format!(
            "{from} has no common commit with {into} - check the name in --branch"
        ));
    }
    let base = merge_base.stdout.trim().to_string();
    let range = format!("{base}..{from}");
    let mut count = vec!["rev-list", "--count", &range];
    count.extend(&pathspec);
    let commits = git::text(root, &count)?.parse::<usize>().unwrap_or(0);
    if commits == 0 {
        return Err(format!(
            "there is no commit on {from} after {into} - commit your work first. Run todoer pr submit in the checkout of the branch that the pull request goes into, with --branch <the branch of the work>"
        ));
    }
    let names: Vec<String> = open.iter().map(|pr| pr.name.clone()).collect();
    let name = options.name.ok_or_else(|| {
        format!(
            "a submit needs the name of the pull request - use -n <name> for a new pull request, or -n <pr> --update to replace an open one. {}",
            open_names(&into, &names)
        )
    })?;
    let tree = format!("{from}^{{tree}}");
    let squashed = git::check_input(
        root,
        &["commit-tree", &tree, "-p", &base, "-F", "-"],
        &format!("{}\n", merge_message(name)),
    )?
    .stdout
    .trim()
    .to_string();
    let mut format_patch = vec![
        "format-patch",
        "--stdout",
        "--no-signature",
        "-1",
        &squashed,
    ];
    format_patch.extend(&pathspec);
    let patch = git::check(root, &format_patch)?.stdout;
    let mut diff_args = vec!["diff", "-M", &base, &from];
    diff_args.extend(&pathspec);
    let diff_text = git::check(root, &diff_args)?.stdout;
    if diff_text.trim().is_empty() {
        return Err(format!(
            "the commits on {from} change no file outside {}/",
            pr_dir_path(name)
        ));
    }
    let existing = open.iter().find(|pr| pr.name == name);
    let cleared = match (existing, options.update) {
        (Some(pr), true) => Some(pr.review()),
        (Some(_), false) => {
            return Err(format!(
                "{into} already has the pull request {name} - use -n {name} --update to replace it, or another -n to make a new pull request"
            ));
        }
        (None, true) => {
            return Err(format!(
                "{into} has no pull request {name} - remove --update to make it. {}",
                open_names(&into, &names)
            ));
        }
        (None, false) => None,
    };
    let created = existing.is_none();
    let mut kept_description = false;
    let description = match (options.message, existing) {
        (Some(message), _) => message.trim().to_string(),
        (None, Some(old)) => {
            kept_description = !old.description().is_empty();
            old.description().to_string()
        }
        (None, None) => String::new(),
    };
    let mut doc = Document::new();
    if !description.is_empty() {
        doc.body = format!("\n{description}\n");
    }
    let mut pr = Pr {
        name: name.to_string(),
        dir: prs_dir(project).join(name),
        author: git::user_email(root)?,
        base,
        todos: removed_todos(&diff::parse(&diff_text)),
        doc,
    };
    let pr_md = pr.render();
    write_file(&pr.dir.join(PATCH), &patch)?;
    write_file(&pr.dir.join(DIFF), &diff_text)?;
    write_file(&pr.dir.join("pr.md"), &pr_md)?;
    if pr.dir.join(REVIEW).exists() {
        std::fs::remove_file(pr.dir.join(REVIEW))
            .map_err(|error| format!("cannot remove {}: {error}", pr.rel(REVIEW)))?;
    }
    let files = diff::parse(&diff_text).len();
    Ok(Submitted {
        pr,
        into,
        from,
        created,
        kept_description,
        commits,
        files,
        cleared,
    })
}

fn apply_message(output: &git::Output) -> String {
    let lines: Vec<&str> = output
        .stdout
        .lines()
        .chain(output.stderr.lines())
        .filter(|line| {
            line.starts_with("CONFLICT")
                || line.starts_with("error:")
                || line.starts_with("Patch failed")
                || line.contains("with conflicts")
        })
        .collect();
    if lines.is_empty() {
        output.message()
    } else {
        lines.join("\n")
    }
}

pub struct Merged {
    pub applied: bool,
    pub done: Vec<String>,
    pub message: String,
}

fn check_no_comments(project: &Project, pr: &Pr) -> Result<(), String> {
    let comments = pr.review().comments;
    if comments.is_empty() {
        return Ok(());
    }
    let ids: Vec<&str> = comments.iter().map(|comment| comment.id.as_str()).collect();
    Err(format!(
        "{} has the unresolved comments {} - the author must do what they ask, then run todoer pr submit --branch <the branch of the work> -n {} --update on {}",
        pr.name,
        ids.join(", "),
        pr.name,
        project.branch_name()?
    ))
}

fn lacks_base(project: &Project, pr: &Pr, command: &str) -> Result<Option<String>, String> {
    let base = format!("{}^{{commit}}", pr.base);
    if git::run(&project.root, &["cat-file", "-e", &base])?.ok {
        return Ok(None);
    }
    Ok(Some(format!(
        "the patch does not apply on {} - nothing changed. A three-way merge needs the base commit {} of {}, and this repository does not have it, for example in a shallow clone. Get the history with git fetch --unshallow, then run {command} again",
        project.branch_name()?,
        pr.base,
        pr.name,
    )))
}

fn does_not_merge(project: &Project, pr: &Pr, output: &git::Output) -> Result<String, String> {
    if let Some(message) = lacks_base(project, pr, &format!("todoer pr apply {} --merge", pr.name))?
    {
        return Ok(message);
    }
    let branch = project.branch_name()?;
    Ok(format!(
        "the patch does not apply on {branch} - nothing changed. To update the pull request, make a new branch from {branch}, run todoer pr apply {name} there, resolve the conflicts and commit, then run todoer pr submit --branch <the new branch> -n {name} --update on {branch}:\n{}",
        apply_message(output),
        name = pr.name,
    ))
}

/// The arguments of `git apply` for the patch of the pull request. The patch removes its todos, but
/// `git apply` leaves them out: an edit of a todo on the current branch must not stop the patch.
/// `remove_todos` removes them by name.
fn apply_args<'a>(pr: &'a Pr, options: &[&'a str]) -> Vec<String> {
    let mut args: Vec<String> = std::iter::once("apply")
        .chain(options.iter().copied())
        .map(str::to_string)
        .collect();
    args.extend(
        pr.todos
            .iter()
            .map(|name| format!("--exclude={}", todo_path(name))),
    );
    args.push(pr.dir.join(PATCH).to_string_lossy().into_owned());
    args
}

fn git_apply(root: &Path, pr: &Pr, options: &[&str]) -> Result<git::Output, String> {
    let args = apply_args(pr, options);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    git::run(root, &args)
}

/// Removes the todos of the pull request that the current checkout has, and stages the removal.
fn remove_todos(project: &Project, lock: &Lock, pr: &Pr) -> Result<Vec<String>, String> {
    let mut done = Vec::new();
    for name in &pr.todos {
        if todo_file(&project.root, name).is_file() {
            remove_todo(&project.root, name)?;
            done.push(name.clone());
        }
    }
    let paths: Vec<String> = pr.todos.iter().map(|name| todo_path(name)).collect();
    project.stage(lock, &paths)?;
    Ok(done)
}

fn merge_into_working_tree(project: &Project, pr: &Pr) -> Result<(), String> {
    let root = &project.root;
    let index = git::git_path(root, "index")?;
    let trial = git::git_path(root, "todoer-trial-index")?;
    if index.is_file() {
        std::fs::copy(&index, &trial)
            .map_err(|error| format!("cannot copy the git index: {error}"))?;
    }
    let args = apply_args(pr, &["--cached", "--3way"]);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let tried = git::run_with_index(root, &args, &trial);
    let _ = std::fs::remove_file(&trial);
    let tried = tried?;
    if !tried.ok {
        return Err(does_not_merge(project, pr, &tried)?);
    }
    let applied = git_apply(root, pr, &["--3way"])?;
    if !applied.ok {
        return Err(does_not_merge(project, pr, &applied)?);
    }
    Ok(())
}

/// Applies the patch, removes the folder and the todos of the pull request, and stages these
/// changes. The person commits the merge.
pub fn merge(project: &Project, pr: &Pr) -> Result<Merged, String> {
    let root = &project.root;
    let lock = project.lock()?;
    check_no_comments(project, pr)?;
    let already = git_apply(root, pr, &["--reverse", "--check"])?.ok;
    if !already {
        merge_into_working_tree(project, pr)?;
    }
    std::fs::remove_dir_all(&pr.dir)
        .map_err(|error| format!("cannot remove {}: {error}", pr.rel_dir()))?;
    project.stage(&lock, &[pr.rel_dir()])?;
    let done = remove_todos(project, &lock, pr)?;
    Ok(Merged {
        applied: !already,
        done,
        message: merge_message(&pr.name),
    })
}

pub enum Applied {
    Already,
    Clean,
    Conflicts(Vec<String>),
}

pub struct ApplyResult {
    pub applied: Applied,
    pub done: Vec<String>,
}

/// Applies the patch to the working tree and the index of the current checkout, with conflict
/// markers where the branch moved. Then a person can run the code, or continue the work and submit
/// with `--update`.
pub fn apply(project: &Project, pr: &Pr) -> Result<ApplyResult, String> {
    let root = &project.root;
    let lock = project.lock()?;
    let branch = project.branch_name()?;
    check_clean(project, pr, &branch)?;
    let applied = if git_apply(root, pr, &["--reverse", "--check"])?.ok {
        Applied::Already
    } else {
        apply_with_conflicts(project, pr, &branch)?
    };
    let done = remove_todos(project, &lock, pr)?;
    Ok(ApplyResult { applied, done })
}

fn apply_with_conflicts(project: &Project, pr: &Pr, branch: &str) -> Result<Applied, String> {
    let root = &project.root;
    let applied = git_apply(root, pr, &["--3way"])?;
    if applied.ok {
        return Ok(Applied::Clean);
    }
    let conflicts: Vec<String> = git::text(root, &["diff", "--name-only", "--diff-filter=U"])?
        .lines()
        .map(str::to_string)
        .collect();
    if conflicts.is_empty() {
        if let Some(message) = lacks_base(project, pr, &format!("todoer pr apply {}", pr.name))? {
            return Err(message);
        }
        return Err(format!(
            "the patch does not apply on {branch} - nothing changed:\n{}",
            apply_message(&applied)
        ));
    }
    Ok(Applied::Conflicts(conflicts))
}

pub fn close(project: &Project, pr: &Pr) -> Result<(), String> {
    let _lock = project.lock()?;
    std::fs::remove_dir_all(&pr.dir)
        .map_err(|error| format!("cannot remove {}: {error}", pr.rel_dir()))
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
