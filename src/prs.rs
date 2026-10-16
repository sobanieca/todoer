//! Pull requests: a folder `.todoer/prs/<name>/` with `pr.md`, the patch, the diff and the review
//! of the latest submit. A submit writes the folder on the branch that the pull request goes into.

use crate::diff;
use crate::frontmatter::Document;
use crate::git;
use crate::pr_view;
use crate::project::{CommitPlan, DATA_DIR, Project, read_file, write_file};
use crate::review::{self, Comment, Review, Target, Tree};
use crate::todos::{
    Todos, read_todo, remove_dependency_on, remove_label_dependencies_without_todos, remove_todo,
    slug, todo_path, valid_name,
};
use std::path::{Path, PathBuf};

const EXCLUDE_DATA: [&str; 3] = ["--", ".", ":(exclude).todoer"];
pub const REVIEW: &str = "review.md";
pub const PATCH: &str = "pr.patch";
pub const DIFF: &str = "pr.diff";
const UPDATE_HEADING: &str = "## Update";

#[derive(Debug, Clone)]
pub struct Pr {
    pub name: String,
    pub dir: PathBuf,
    pub author: String,
    pub from: String,
    pub into: String,
    pub base: String,
    pub todos: Vec<String>,
    pub doc: Document,
}

pub fn prs_dir(project: &Project) -> PathBuf {
    project.data_dir().join("prs")
}

impl Pr {
    fn from_text(dir: &Path, name: &str, text: &str) -> Pr {
        let doc = Document::parse(text);
        Pr {
            name: name.to_string(),
            dir: dir.to_path_buf(),
            author: doc.text("author").unwrap_or_default(),
            from: doc.text("from").unwrap_or_default(),
            into: doc.text("into").unwrap_or_default(),
            base: doc.text("base").unwrap_or_default(),
            todos: doc.list("todos"),
            doc,
        }
    }

    fn load(dir: &Path, name: &str) -> Result<Pr, String> {
        Ok(Pr::from_text(dir, name, &read_file(&dir.join("pr.md"))?))
    }

    fn render(&mut self) -> String {
        self.doc.set("author", &self.author);
        self.doc.set("from", &self.from);
        self.doc.set("into", &self.into);
        self.doc.set("base", &self.base);
        self.doc.set_list("todos", &self.todos);
        self.doc.render()
    }

    pub fn description(&self) -> &str {
        self.doc.body.trim()
    }

    pub fn rel_dir(&self) -> String {
        format!("{DATA_DIR}/prs/{}", self.name)
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
    if !valid_name(&name) || !dir.join("pr.md").is_file() {
        return Err(format!(
            "there is no open pull request {name} - run todoer pr ls"
        ));
    }
    Pr::load(&dir, &name)
}

pub enum Destination {
    Checkout(Project),
    Branch(String),
}

pub struct Submitted {
    pub pr: Pr,
    pub created: bool,
    pub commits: usize,
    pub files: usize,
    pub cleared: Option<Review>,
    pub destination: Destination,
    pub plan: CommitPlan,
}

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

pub struct SubmitOptions<'a> {
    pub branch: Option<&'a str>,
    pub name: Option<&'a str>,
    pub message: Option<&'a str>,
    pub todos: &'a [String],
    pub update: bool,
    pub no_commit: bool,
}

fn check_name(name: &str) -> Result<(), String> {
    if valid_name(name) {
        return Ok(());
    }
    match slug(name, 60) {
        good if good.is_empty() => Err(format!(
            "\"{name}\" is not a pull request name - use lowercase letters, digits and hyphens"
        )),
        good => Err(format!(
            "\"{name}\" is not a pull request name - use {good}"
        )),
    }
}

fn free_name(root: &Path, taken: &[String], branch: &str, range: &str) -> Result<String, String> {
    let mut log = vec!["log", "--reverse", "--format=%s", range];
    log.extend(EXCLUDE_DATA);
    let first_subject = git::text(root, &log)?
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    let base_name = match slug(&first_subject, 40) {
        name if name.is_empty() => slug(branch, 40),
        name => name,
    };
    let base_name = if base_name.is_empty() {
        "pr".to_string()
    } else {
        base_name
    };
    let mut name = base_name.clone();
    let mut number = 2;
    while taken.contains(&name) {
        name = format!("{base_name}-{number}");
        number += 1;
    }
    Ok(name)
}

fn squash_message(root: &Path, range: &str) -> Result<String, String> {
    let mut log = vec!["log", "--reverse", "--format=%B%x00", range];
    log.extend(EXCLUDE_DATA);
    let text = git::text(root, &log)?;
    let messages: Vec<&str> = text
        .split('\0')
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .collect();
    let subjects: Vec<&str> = messages
        .iter()
        .map(|message| message.lines().next().unwrap_or(""))
        .collect();
    Ok(match messages.as_slice() {
        [] => "todoer: squash".to_string(),
        [message] => format!("{message}\n"),
        _ => {
            let list: Vec<String> = subjects
                .iter()
                .map(|subject| format!("* {subject}"))
                .collect();
            format!("{}\n\n{}\n", subjects[0], list.join("\n"))
        }
    })
}

impl Destination {
    fn find(project: &Project, branch: &str) -> Result<Destination, String> {
        match git::checkout_of_branch(&project.root, branch)? {
            Some(root) => Ok(Destination::Checkout(Project::find_from(&root)?)),
            None => Ok(Destination::Branch(branch.to_string())),
        }
    }

    pub fn checkout(&self) -> Option<&Path> {
        match self {
            Destination::Checkout(project) => Some(&project.root),
            Destination::Branch(_) => None,
        }
    }

    fn prs_dir(&self, project: &Project) -> PathBuf {
        match self {
            Destination::Checkout(target) => prs_dir(target),
            Destination::Branch(_) => prs_dir(project),
        }
    }

    fn open_prs(&self, project: &Project) -> Result<Vec<(Pr, Review)>, String> {
        let branch = match self {
            Destination::Checkout(target) => {
                return Ok(list(target)?
                    .into_iter()
                    .map(|pr| {
                        let review = pr.review();
                        (pr, review)
                    })
                    .collect());
            }
            Destination::Branch(branch) => branch,
        };
        let root = &project.root;
        let tree = format!("refs/heads/{branch}");
        let folders = git::text(
            root,
            &["ls-tree", "--name-only", &tree, &format!("{DATA_DIR}/prs/")],
        )?;
        let mut open = Vec::new();
        for folder in folders.lines() {
            let name = folder.rsplit('/').next().unwrap_or(folder);
            let Some(text) = git::show(root, &tree, &format!("{folder}/pr.md"))? else {
                continue;
            };
            let review = git::show(root, &tree, &format!("{folder}/{REVIEW}"))?
                .map(|text| Review::parse(&text))
                .unwrap_or_else(Review::empty);
            let pr = Pr::from_text(&prs_dir(project).join(name), name, &text);
            open.push((pr, review));
        }
        Ok(open)
    }
}

fn check_clean(project: &Project, branch: &str) -> Result<(), String> {
    let others = project.plan_commit(false, &[DATA_DIR.to_string()])?.others;
    if others.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{branch} has changes that are not committed: {} - commit them first",
        others.join(", ")
    ))
}

pub fn submit(project: &Project, options: &SubmitOptions) -> Result<Submitted, String> {
    let root = &project.root;
    let branch = git::branch(root)?
        .ok_or("the current checkout has no branch - check out the branch of your work")?;
    let into = options
        .branch
        .filter(|into| !into.trim().is_empty())
        .ok_or("a submit needs the branch that the pull request goes into - use todoer pr submit --branch <branch>")?
        .to_string();
    if into == branch {
        return Err(format!(
            "the pull request cannot go into its own branch {branch} - use --branch <the branch that it goes into>"
        ));
    }
    if !git::local_branch_exists(root, &into)? {
        return Err(format!(
            "there is no local branch {into} - check the name in --branch"
        ));
    }
    for name in options.todos {
        read_todo(root, name)?;
    }
    check_clean(project, &branch)?;
    let name = options.name.filter(|name| !name.trim().is_empty());
    if let Some(name) = name {
        check_name(name)?;
    }
    let lock = project.lock()?;
    let destination = Destination::find(project, &into)?;
    if options.no_commit && destination.checkout().is_none() {
        return Err(format!(
            "{into} is not checked out, so todoer can only commit the pull request on it - check out {into}, or remove --no-commit"
        ));
    }
    let open = destination.open_prs(project)?;
    let merge_base = git::run(root, &["merge-base", "HEAD", &into])?;
    if !merge_base.ok {
        return Err(format!(
            "{branch} has no common commit with {into} - check the name in --branch"
        ));
    }
    let base = merge_base.stdout.trim().to_string();
    let range = format!("{base}..HEAD");
    let mut count = vec!["rev-list", "--count", &range];
    count.extend(EXCLUDE_DATA);
    let commits = git::text(root, &count)?.parse::<usize>().unwrap_or(0);
    if commits == 0 {
        return Err(format!(
            "there is no commit on {branch} after {into} - commit your work first"
        ));
    }
    let squashed = git::check_input(
        root,
        &["commit-tree", "HEAD^{tree}", "-p", &base, "-F", "-"],
        &squash_message(root, &range)?,
    )?
    .stdout
    .trim()
    .to_string();
    let mut format_patch = vec!["format-patch", "--stdout", "-1", &squashed];
    format_patch.extend(EXCLUDE_DATA);
    let patch = git::check(root, &format_patch)?.stdout;
    let mut diff_args = vec!["diff", "-M", &base, "HEAD"];
    diff_args.extend(EXCLUDE_DATA);
    let diff_text = git::check(root, &diff_args)?.stdout;
    if diff_text.trim().is_empty() {
        return Err(format!(
            "the commits on {branch} change no file outside {DATA_DIR}/"
        ));
    }
    let author = git::user_email(root)?;
    let of_branch = open.iter().find(|(pr, _)| pr.from == branch);
    let existing = match (name, of_branch) {
        (Some(name), Some((pr, _))) if pr.name != name => {
            return Err(format!(
                "the pull request of {branch} is {} - a submit cannot rename it to {name}",
                pr.name
            ));
        }
        (Some(name), None) => match open.iter().find(|(pr, _)| pr.name == name) {
            Some(_) if !options.update => {
                return Err(format!(
                    "the pull request {name} exists - pick another name, or use --update to replace its patch"
                ));
            }
            found => found,
        },
        (_, found) => found,
    };
    let created = existing.is_none();
    let cleared = match existing {
        Some((pr, review)) => {
            if !review.comments.is_empty() && !options.update {
                return Err(format!(
                    "{} is already submitted, and its review has {} - do what the comments ask, then run todoer pr submit --update to replace the patch and remove the review",
                    pr.name,
                    plural(review.comments.len(), "comment")
                ));
            }
            Some(review.clone())
        }
        None => None,
    };
    let target_dir = destination.prs_dir(project);
    let mut pr = match existing {
        Some((pr, _)) => {
            let mut pr = pr.clone();
            pr.dir = target_dir.join(&pr.name);
            pr
        }
        None => {
            let taken: Vec<String> = open.iter().map(|(pr, _)| pr.name.clone()).collect();
            let name = match name {
                Some(name) => name.to_string(),
                None => free_name(root, &taken, &branch, &range)?,
            };
            Pr {
                name: name.clone(),
                dir: target_dir.join(&name),
                author: author.clone(),
                from: String::new(),
                into: String::new(),
                base: String::new(),
                todos: Vec::new(),
                doc: Document::new(),
            }
        }
    };
    pr.from = branch;
    pr.into = into.clone();
    pr.base = base;
    for name in options.todos {
        if !pr.todos.contains(name) {
            pr.todos.push(name.clone());
        }
    }
    let message = options
        .message
        .map(str::trim)
        .filter(|text| !text.is_empty());
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
    let pr_md = pr.render();
    let action = if created { "submit" } else { "update" };
    let commit_message = format!("todoer: {action} {}", pr.name);
    let plan = match &destination {
        Destination::Checkout(target) => {
            let own = [pr.rel_dir()];
            let plan = target.plan_commit(options.no_commit, &own)?;
            write_file(&pr.dir.join(PATCH), &patch)?;
            write_file(&pr.dir.join(DIFF), &diff_text)?;
            write_file(&pr.dir.join("pr.md"), &pr_md)?;
            if pr.dir.join(REVIEW).exists() {
                std::fs::remove_file(pr.dir.join(REVIEW))
                    .map_err(|error| format!("cannot remove {}: {error}", pr.rel(REVIEW)))?;
            }
            if plan.commit {
                let committed = target.commit(&lock, &commit_message, &own);
                if let Err(error) = committed {
                    if created {
                        let _ = std::fs::remove_dir_all(&pr.dir);
                    }
                    return Err(error);
                }
            }
            plan
        }
        Destination::Branch(into) => {
            let files = [
                (pr.rel("pr.md"), pr_md),
                (pr.rel(PATCH), patch),
                (pr.rel(DIFF), diff_text.clone()),
            ];
            project.commit_to_branch(&lock, into, &commit_message, &pr.rel_dir(), &files)?;
            CommitPlan {
                commit: true,
                others: Vec::new(),
            }
        }
    };
    let files = diff::parse(&diff_text).len();
    Ok(Submitted {
        pr,
        created,
        commits,
        files,
        cleared,
        destination,
        plan,
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
    pub changed: Vec<String>,
    pub removed_labels: Vec<(String, String)>,
    pub plan: CommitPlan,
}

fn check_no_comments(pr: &Pr) -> Result<(), String> {
    let comments = pr.review().comments;
    if comments.is_empty() {
        return Ok(());
    }
    let ids: Vec<&str> = comments.iter().map(|comment| comment.id.as_str()).collect();
    Err(format!(
        "{} has the unresolved comments {} - the author must do what they ask, then run todoer pr submit --update",
        pr.name,
        ids.join(", ")
    ))
}

fn does_not_apply(project: &Project, output: &git::Output) -> Result<String, String> {
    Ok(format!(
        "the patch does not apply on {} - nothing changed. Ask the author to rebase the branch on {} and run todoer pr submit --update:\n{}",
        project.branch_name()?,
        project.branch_name()?,
        apply_message(output)
    ))
}

fn apply_to_working_tree(project: &Project, patch: &str) -> Result<(), String> {
    let root = &project.root;
    let index = git::git_path(root, "index")?;
    let trial = git::git_path(root, "todoer-trial-index")?;
    if index.is_file() {
        std::fs::copy(&index, &trial)
            .map_err(|error| format!("cannot copy the git index: {error}"))?;
    }
    let tried = git::run_with_index(root, &["apply", "--cached", "--3way", patch], &trial);
    let _ = std::fs::remove_file(&trial);
    let tried = tried?;
    if !tried.ok {
        return Err(does_not_apply(project, &tried)?);
    }
    let applied = git::run(root, &["apply", "--3way", patch])?;
    if !applied.ok {
        return Err(does_not_apply(project, &applied)?);
    }
    Ok(())
}

pub fn merge(project: &Project, pr: &Pr, no_commit: bool) -> Result<Merged, String> {
    let root = &project.root;
    let lock = project.lock()?;
    check_no_comments(pr)?;
    let todos = Todos::load(root)?;
    let done: Vec<String> = pr
        .todos
        .iter()
        .filter(|name| todos.get(name).is_some())
        .cloned()
        .collect();
    let mut own = vec![pr.rel_dir()];
    own.extend(done.iter().map(|name| todo_path(name)));
    let plan = project.plan_commit(no_commit, &own)?;
    let patch = pr.dir.join(PATCH).to_string_lossy().into_owned();
    let already = git::run(root, &["apply", "--reverse", "--check", &patch])?.ok;
    if !already && plan.commit {
        let result = git::run(root, &["am", "-q", "--3way", &patch])?;
        if !result.ok {
            let _ = git::run(root, &["am", "--abort"]);
            return Err(does_not_apply(project, &result)?);
        }
    } else if !already {
        apply_to_working_tree(project, &patch)?;
    }
    let mut changed = Vec::new();
    for name in &done {
        remove_todo(root, name)?;
    }
    for name in &done {
        let todos = Todos::load(root)?;
        for other in remove_dependency_on(root, &todos, name)? {
            if !changed.contains(&other) {
                changed.push(other);
            }
        }
    }
    let removed_labels = remove_label_dependencies_without_todos(root, &Todos::load(root)?)?;
    let mut paths = changed.clone();
    for (name, _) in &removed_labels {
        if !paths.contains(name) {
            paths.push(name.clone());
        }
    }
    std::fs::remove_dir_all(&pr.dir)
        .map_err(|error| format!("cannot remove {}: {error}", pr.rel_dir()))?;
    if plan.commit {
        own.extend(paths.iter().map(|name| todo_path(name)));
        if already {
            project.commit(&lock, &format!("todoer: merge {}", pr.name), &own)?;
        } else {
            project.amend(&lock, &own)?;
        }
    }
    Ok(Merged {
        applied: !already,
        done,
        changed,
        removed_labels,
        plan,
    })
}

pub fn close(project: &Project, pr: &Pr, no_commit: bool) -> Result<CommitPlan, String> {
    let lock = project.lock()?;
    let own = [pr.rel_dir()];
    let plan = project.plan_commit(no_commit, &own)?;
    std::fs::remove_dir_all(&pr.dir)
        .map_err(|error| format!("cannot remove {}: {error}", pr.rel_dir()))?;
    if plan.commit {
        project.commit(&lock, &format!("todoer: close {}", pr.name), &own)?;
    }
    Ok(plan)
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
