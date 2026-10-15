//! `ls`, `add`, `done`, `mv`, `check`, `claim` and `unclaim`.

use crate::args::{Args, Flag, option, switch};
use crate::colors;
use crate::frontmatter::Document;
use crate::git;
use crate::json::{self, Value};
use crate::project::{Project, read_file, write_file};
use crate::terminal;
use crate::todos::{Todo, Todos, check_label, read_todo, slug};

pub const LS_FLAGS: &[Flag] = &[
    switch("all", Some('a')),
    switch("mine", None),
    option("label", Some('l')),
    switch("ready", None),
    switch("json", None),
];

pub fn state_color(state: &str, text: &str) -> String {
    match state {
        "in-progress" => colors::yellow(text),
        "ready" => colors::green(text),
        "blocked" => colors::red(text),
        _ => colors::gray(text),
    }
}

fn state_rank(state: &str) -> usize {
    ["in-progress", "ready", "blocked", "done", "dropped"]
        .iter()
        .position(|entry| *entry == state)
        .unwrap_or(5)
}

pub fn ls(args: &Args) -> Result<(), String> {
    args.expect_positional(
        0,
        0,
        "todoer ls [--all] [--mine] [--label l] [--ready] [--json]",
    )?;
    let project = Project::find()?;
    let todos = Todos::load(&project.current)?;
    todos.print_warnings();
    let me = if args.has("mine") {
        Some(project.me()?)
    } else {
        None
    };
    let label = args.value("label");
    let mut rows: Vec<(&Todo, &'static str)> = todos
        .items
        .iter()
        .map(|todo| (todo, todos.state(todo)))
        .filter(|(todo, _)| args.has("all") || !todo.is_closed())
        .filter(|(_, state)| !args.has("ready") || *state == "ready")
        .filter(|(todo, _)| me.is_none() || todo.assignee == me)
        .filter(|(todo, _)| label.is_none_or(|label| todo.labels.iter().any(|l| l == label)))
        .collect();
    rows.sort_by(|a, b| {
        state_rank(a.1)
            .cmp(&state_rank(b.1))
            .then_with(|| a.0.path.cmp(&b.0.path))
    });
    if args.has("json") {
        let items = rows
            .iter()
            .map(|(todo, state)| todo_json(&todos, todo, state))
            .collect();
        println!("{}", json::stringify(&Value::Array(items)));
        return Ok(());
    }
    if rows.is_empty() {
        println!("no todos");
        return Ok(());
    }
    let path_width = rows
        .iter()
        .map(|(todo, _)| todo.path.len())
        .max()
        .unwrap_or(0);
    let title_width = rows
        .iter()
        .map(|(todo, _)| todo.title.chars().count())
        .max()
        .unwrap_or(0)
        .min(50);
    for (todo, state) in rows {
        let extra = match state {
            "blocked" => format!("waits for {}", todos.waits_for(todo).join(", ")),
            _ => todo.assignee.clone().unwrap_or_default(),
        };
        let line = format!(
            "{}  {}  {}  {}",
            state_color(state, &colors::pad(state, 11)),
            colors::pad(&todo.path, path_width),
            colors::pad(&todo.title, title_width),
            colors::gray(&extra)
        );
        println!("{}", line.trim_end());
    }
    Ok(())
}

pub fn todo_json(todos: &Todos, todo: &Todo, state: &str) -> Value {
    Value::object(vec![
        ("path", Value::str(&todo.path)),
        ("title", Value::str(&todo.title)),
        ("status", Value::str(&todo.status)),
        ("state", Value::str(state)),
        ("assignee", Value::opt(todo.assignee.as_deref())),
        ("labels", Value::strings(&todo.labels)),
        ("dependsOn", Value::strings(&todo.depends_on)),
        ("waitsFor", Value::strings(&todos.waits_for(todo))),
    ])
}

pub const ADD_FLAGS: &[Flag] = &[
    option("title", Some('t')),
    option("path", Some('p')),
    option("label", Some('l')),
    option("depends-on", Some('d')),
];

pub fn add(args: &Args) -> Result<(), String> {
    let project = Project::find()?;
    let title = args
        .value("title")
        .map(|title| title.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|title| !title.is_empty())
        .ok_or("a todo needs a title - use todoer add -t \"<title>\" [text]")?;
    let labels = args.values("label");
    for label in &labels {
        check_label(label)?;
    }
    let folder = match args.value("path") {
        Some(dir) => project.relative(dir.trim_end_matches('/')).map(Some)?,
        None => project.relative(".").ok(),
    };
    let mut depends_on = Vec::new();
    let deps = args.values("depends-on");
    if !deps.is_empty() {
        let todos = Todos::load(&project.current)?;
        for dep in deps {
            let path = project.relative(&dep)?;
            todos.require(&path)?;
            depends_on.push(path);
        }
    }
    let text = if !args.positional.is_empty() {
        args.positional.join(" ")
    } else {
        terminal::read_stdin_if_piped()?
    };
    let name = match slug(&title, 60) {
        name if name.is_empty() => "todo".to_string(),
        name => name,
    };
    let prefix = folder.map(|dir| format!("{dir}/")).unwrap_or_default();
    let mut path = format!("{prefix}{name}.md");
    let mut number = 2;
    while project.current.join(&path).exists() {
        path = format!("{prefix}{name}-{number}.md");
        number += 1;
    }
    let mut doc = Document::new();
    doc.set_text("title", &title);
    doc.set("status", "todo");
    doc.set_list("labels", &labels);
    doc.set_list("dependsOn", &depends_on);
    let text = text.trim();
    doc.body = if text.is_empty() {
        String::new()
    } else {
        format!("\n{text}\n")
    };
    write_file(&project.current.join(&path), &doc.render())?;
    println!("{path}");
    Ok(())
}

pub fn done(args: &Args) -> Result<(), String> {
    if args.positional.is_empty() {
        return Err("use: todoer done <path> [<path> ...]".into());
    }
    let project = Project::find()?;
    for argument in &args.positional {
        let path = project.relative(argument)?;
        let mut todo = read_todo(&project.current, &path)?;
        todo.set_status("done");
        todo.save(&project.current)?;
        println!("done: {path}");
    }
    Ok(())
}

pub fn mv(args: &Args) -> Result<(), String> {
    args.expect_positional(2, 2, "todoer mv <old> <new>")?;
    let project = Project::find()?;
    let root = &project.current;
    let old = project.relative(&args.positional[0])?;
    let mut new = project.relative(&args.positional[1])?;
    if args.positional[1].ends_with('/') || root.join(&new).is_dir() {
        let name = old.rsplit('/').next().unwrap_or(&old);
        new = format!("{new}/{name}");
    }
    if !new.ends_with(".md") {
        return Err(format!("{new} must end with .md"));
    }
    let todos = Todos::load(root)?;
    todos.require(&old)?;
    if root.join(&new).exists() {
        return Err(format!("{new} exists - pick another path"));
    }
    if let Some(parent) = root.join(&new).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("cannot make {}: {error}", parent.display()))?;
    }
    if git::is_tracked(root, &old)? {
        git::check(root, &["mv", "--", &old, &new])?;
    } else {
        std::fs::rename(root.join(&old), root.join(&new))
            .map_err(|error| format!("cannot move {old}: {error}"))?;
    }
    println!("moved {old} -> {new}");
    for todo in &todos.items {
        if !todo.depends_on.contains(&old) {
            continue;
        }
        let path = if todo.path == old { &new } else { &todo.path };
        let mut changed = read_todo(root, path)?;
        let deps = changed
            .depends_on
            .iter()
            .map(|dep| {
                if dep == &old {
                    new.clone()
                } else {
                    dep.clone()
                }
            })
            .collect();
        changed.set_depends_on(deps);
        changed.save(root)?;
        println!("updated dependsOn in {path}");
    }
    Ok(())
}

pub fn check(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 0, "todoer check")?;
    let project = Project::find()?;
    let todos = Todos::load(&project.current)?;
    let problems = todos.problems();
    for problem in &problems {
        println!("{problem}");
    }
    if problems.is_empty() {
        println!("{} todos, no problems", todos.items.len());
        Ok(())
    } else {
        Err(format!(
            "{} problem{} in {} todos",
            problems.len(),
            if problems.len() == 1 { "" } else { "s" },
            todos.items.len()
        ))
    }
}

pub const CLAIM_FLAGS: &[Flag] = &[switch("force", None)];

fn main_checkout_todo(project: &Project, path: &str) -> Result<(Todo, String), String> {
    let root = &project.root;
    if git::is_tracked(root, path)? && git::has_changes(root, path)? {
        return Err(format!(
            "{path} has uncommitted changes in the main checkout {} - commit or discard them first",
            root.display()
        ));
    }
    if !root.join(path).is_file() {
        return Err(format!(
            "there is no todo {path} in the main checkout {} - commit it on {} first",
            root.display(),
            project.main_branch
        ));
    }
    let original = read_file(&root.join(path))?;
    Ok((read_todo(root, path)?, original))
}

fn commit_todo(
    project: &Project,
    lock: &crate::project::Lock,
    todo: &Todo,
    original: &str,
    message: &str,
) -> Result<(), String> {
    todo.save(&project.root)?;
    let result = project.commit_on_main(lock, message, std::slice::from_ref(&todo.path));
    if result.is_err() {
        let _ = write_file(&project.root.join(&todo.path), original);
    }
    result
}

pub fn claim(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, "todoer claim <path> [--force]")?;
    let project = Project::find()?;
    let path = project.relative(&args.positional[0])?;
    let me = project.me()?;
    let lock = project.lock()?;
    project.require_main_checkout_on_main()?;
    let (mut todo, original) = main_checkout_todo(&project, &path)?;
    match (todo.status.as_str(), todo.assignee.as_deref()) {
        ("in-progress", Some(assignee)) if assignee == me => {
            println!("{path} is already claimed by you");
            return Ok(());
        }
        ("in-progress", Some(assignee)) => {
            return Err(format!(
                "{path} is claimed by {assignee} - pick another todo, or ask {assignee} to run todoer unclaim"
            ));
        }
        ("done" | "dropped", _) => return Err(format!("{path} is {}", todo.status)),
        _ => {}
    }
    let todos = Todos::load(&project.root)?;
    let waits = todos.waits_for(&todo);
    if !waits.is_empty() && !args.has("force") {
        return Err(format!(
            "{path} waits for {} - claim a todo that is ready, or use --force",
            waits.join(", ")
        ));
    }
    todo.set_status("in-progress");
    todo.set_assignee(Some(&me));
    commit_todo(
        &project,
        &lock,
        &todo,
        &original,
        &format!("todoer: claim {path}"),
    )?;
    println!("claimed {path} for {me} on {}", project.main_branch);
    Ok(())
}

pub fn unclaim(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, "todoer unclaim <path>")?;
    let project = Project::find()?;
    let path = project.relative(&args.positional[0])?;
    let lock = project.lock()?;
    project.require_main_checkout_on_main()?;
    let (mut todo, original) = main_checkout_todo(&project, &path)?;
    if todo.status != "in-progress" {
        return Err(format!(
            "{path} is not claimed - its status is {}",
            todo.status
        ));
    }
    let previous = todo.assignee.clone().unwrap_or_else(|| "nobody".into());
    todo.set_status("todo");
    todo.set_assignee(None);
    commit_todo(
        &project,
        &lock,
        &todo,
        &original,
        &format!("todoer: unclaim {path}"),
    )?;
    println!("unclaimed {path} (was {previous})");
    Ok(())
}
