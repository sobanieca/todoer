//! `ls`, `add`, `done`, `mv`, `check`, `claim` and `unclaim`. Only `claim` and `unclaim` commit.

use crate::args::{Args, Flag, option, switch};
use crate::colors;
use crate::commands::report;
use crate::frontmatter::Document;
use crate::git;
use crate::json::{self, Value};
use crate::project::{CommitPlan, Lock, Project, write_file};
use crate::rules::Rules;
use crate::terminal;
use crate::todos::{
    Todo, Todos, check_label, check_name, name_of, read_todo, remove_dependency_on,
    remove_label_dependencies_without_todos, remove_todo, todo_file, todo_path, todos_dir,
};
use std::path::{Path, PathBuf};

pub const LS_FLAGS: &[Flag] = &[
    switch("mine", None),
    option("label", Some('l')),
    switch("available", None),
    switch("json", None),
];

pub fn state_color(state: &str, text: &str) -> String {
    match state {
        "in-progress" => colors::yellow(text),
        "available" => colors::green(text),
        "blocked" => colors::red(text),
        _ => colors::gray(text),
    }
}

fn state_rank(state: &str) -> usize {
    ["in-progress", "available", "blocked"]
        .iter()
        .position(|entry| *entry == state)
        .unwrap_or(3)
}

struct Listing<'a> {
    todos: &'a Todos,
    rows: Vec<(&'a Todo, &'static str)>,
}

fn select<'a>(todos: &'a Todos, args: &Args, me: Option<&str>) -> Listing<'a> {
    let label = args.value("label");
    let mut rows: Vec<(&Todo, &'static str)> = todos
        .items
        .iter()
        .map(|todo| (todo, todos.state(todo)))
        .filter(|(_, state)| !args.has("available") || *state == "available")
        .filter(|(todo, _)| me.is_none() || todo.assignee.as_deref() == me)
        .filter(|(todo, _)| label.is_none_or(|label| todo.labels.iter().any(|l| l == label)))
        .collect();
    rows.sort_by(|a, b| {
        state_rank(a.1)
            .cmp(&state_rank(b.1))
            .then_with(|| a.0.name.cmp(&b.0.name))
    });
    Listing { todos, rows }
}

fn print_rows(listing: &Listing, indent: &str) {
    let width = listing
        .rows
        .iter()
        .map(|(todo, _)| todo.name.len())
        .max()
        .unwrap_or(0);
    for (todo, state) in &listing.rows {
        let extra = match *state {
            "blocked" => format!("waits for {}", listing.todos.waits_summary(todo).join(", ")),
            _ => todo.assignee.clone().unwrap_or_default(),
        };
        let line = format!(
            "{indent}{}  {}  {}",
            state_color(state, &colors::pad(state, 11)),
            colors::pad(&todo.name, width),
            colors::gray(&extra)
        );
        println!("{}", line.trim_end());
    }
}

pub fn todo_json(todos: &Todos, todo: &Todo, state: &str) -> Vec<(&'static str, Value)> {
    vec![
        ("name", Value::str(&todo.name)),
        ("state", Value::str(state)),
        ("assignee", Value::opt(todo.assignee.as_deref())),
        ("labels", Value::strings(&todo.labels)),
        ("dependsOn", Value::strings(&todo.depends_on)),
        ("dependsOnLabels", Value::strings(&todo.depends_on_labels)),
        ("waitsFor", Value::strings(&todos.waits_for(todo))),
    ]
}

const LS_USAGE: &str = "todoer ls [--mine] [--label l] [--available] [--json]";

pub fn ls(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 0, LS_USAGE)?;
    let cwd = std::env::current_dir()
        .map_err(|error| format!("cannot read the current folder: {error}"))?;
    if git::toplevel(&cwd).is_none() {
        return ls_projects(args, &cwd);
    }
    let project = Project::find_from(&cwd)?;
    let todos = Todos::load(&project.root)?;
    todos.print_warnings();
    let me = if args.has("mine") {
        Some(project.me()?)
    } else {
        None
    };
    let listing = select(&todos, args, me.as_deref());
    if args.has("json") {
        let items = listing
            .rows
            .iter()
            .map(|(todo, state)| Value::object(todo_json(&todos, todo, state)))
            .collect();
        println!("{}", json::stringify(&Value::Array(items)));
        return Ok(());
    }
    if listing.rows.is_empty() {
        println!("no todos");
        return Ok(());
    }
    print_rows(&listing, "");
    Ok(())
}

fn projects_in(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.join(".git").exists() && todos_dir(path).is_dir())
        .collect();
    found.sort();
    found
}

fn ls_projects(args: &Args, folder: &Path) -> Result<(), String> {
    let projects = projects_in(folder);
    if projects.is_empty() {
        return Err(
            "this folder is not in a git repository, and no folder in it has todos - run todoer in a project"
                .to_string(),
        );
    }
    let mut items = Vec::new();
    let mut printed = false;
    for root in &projects {
        let name = root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let branch = Project::find_from(root)?.branch_name()?;
        crate::logger::warn(&format!(
            "{name}: todoer read the todos of the checked-out branch {branch}"
        ));
        let todos = Todos::load(root)?;
        for problem in todos.problems() {
            crate::logger::warn(&format!("{name}: {problem}"));
        }
        let me = if args.has("mine") {
            Some(git::user_email(root)?)
        } else {
            None
        };
        let listing = select(&todos, args, me.as_deref());
        if args.has("json") {
            for (todo, state) in &listing.rows {
                let mut fields = vec![
                    ("project", Value::str(&name)),
                    ("branch", Value::str(&branch)),
                ];
                fields.extend(todo_json(&todos, todo, state));
                items.push(Value::object(fields));
            }
            continue;
        }
        if listing.rows.is_empty() {
            continue;
        }
        if printed {
            println!();
        }
        println!("{}  {}", colors::bold(&name), colors::gray(&branch));
        print_rows(&listing, "  ");
        printed = true;
    }
    if args.has("json") {
        println!("{}", json::stringify(&Value::Array(items)));
    } else if !printed {
        println!("no todos");
    }
    Ok(())
}

pub const ADD_FLAGS: &[Flag] = &[
    option("label", Some('l')),
    option("depends-on", Some('d')),
    option("depends-on-label", Some('D')),
];

pub fn add(args: &Args) -> Result<(), String> {
    let usage = "todoer add <name> [-l label]... [-d name]... [-D label]... [text]";
    let Some(name) = args.positional.first() else {
        return Err(format!("a todo needs a name - use {usage}"));
    };
    check_name(name)?;
    let labels = args.values("label");
    for label in &labels {
        check_label(label)?;
    }
    let project = Project::find()?;
    let _lock = project.lock()?;
    let todos = Todos::load(&project.root)?;
    if todos.get(name).is_some() || todo_file(&project.root, name).exists() {
        return Err(format!("the todo {name} exists - pick another name"));
    }
    let mut depends_on = Vec::new();
    for dep in args.values("depends-on") {
        let dep = name_of(&dep);
        todos.require(&dep)?;
        depends_on.push(dep);
    }
    let depends_on_labels = args.values("depends-on-label");
    for label in &depends_on_labels {
        check_label(label)?;
        if todos.labeled_except(label, name).is_empty() {
            return Err(format!("no todo has the label {label} - run todoer labels"));
        }
    }
    let text = if args.positional.len() > 1 {
        args.positional[1..].join(" ")
    } else {
        terminal::read_stdin_if_piped()?
    };
    let mut doc = Document::new();
    doc.set_list("labels", &labels);
    doc.set_list("dependsOn", &depends_on);
    doc.set_list("dependsOnLabels", &depends_on_labels);
    let text = text.trim();
    doc.body = if text.is_empty() {
        String::new()
    } else {
        format!("\n{text}\n")
    };
    todos
        .with(Todo::from_document(name, doc.clone()))
        .check_no_cycle(std::slice::from_ref(name))?;
    write_file(&todo_file(&project.root, name), &doc.render())?;
    println!("{}", todo_path(name));
    if !Rules::load(&project.root)?.items.is_empty() {
        println!("check the label rules: todoer labels");
    }
    Ok(())
}

pub fn done(args: &Args) -> Result<(), String> {
    if args.positional.is_empty() {
        return Err("use: todoer done <name> [<name> ...]".into());
    }
    let project = Project::find()?;
    let _lock = project.lock()?;
    let names: Vec<String> = args.positional.iter().map(|arg| name_of(arg)).collect();
    let todos = Todos::load(&project.root)?;
    for name in &names {
        todos.require(name)?;
    }
    for name in &names {
        remove_todo(&project.root, name)?;
        println!("done: {name}");
    }
    let mut todos = Todos::load(&project.root)?;
    for name in &names {
        for changed in remove_dependency_on(&project.root, &todos, name)? {
            println!("removed {name} from dependsOn in {changed}");
        }
        todos = Todos::load(&project.root)?;
    }
    print_removed_labels(&remove_label_dependencies_without_todos(
        &project.root,
        &todos,
    )?);
    Ok(())
}

pub fn print_removed_labels(removed: &[(String, String)]) {
    for (name, label) in removed {
        println!("removed {label} from dependsOnLabels in {name}: no other todo has the label");
    }
}

pub fn mv(args: &Args) -> Result<(), String> {
    args.expect_positional(2, 2, "todoer mv <old> <new>")?;
    let project = Project::find()?;
    let root = &project.root;
    let old = name_of(&args.positional[0]);
    let new = args.positional[1].clone();
    check_name(&new)?;
    let _lock = project.lock()?;
    let todos = Todos::load(root)?;
    todos.require(&old)?;
    if todo_file(root, &new).exists() {
        return Err(format!("the todo {new} exists - pick another name"));
    }
    std::fs::rename(todo_file(root, &old), todo_file(root, &new))
        .map_err(|error| format!("cannot rename {old}: {error}"))?;
    println!("renamed {old} -> {new}");
    for todo in &todos.items {
        if !todo.depends_on.contains(&old) {
            continue;
        }
        let name = if todo.name == old { &new } else { &todo.name };
        let mut changed = read_todo(root, name)?;
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
        println!("updated dependsOn in {name}");
    }
    Ok(())
}

pub fn check(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 0, "todoer check")?;
    let project = Project::find()?;
    let todos = Todos::load(&project.root)?;
    let rules = Rules::load(&project.root)?;
    let mut problems = todos.problems();
    problems.extend_from_slice(rules.problems());
    for problem in &problems {
        println!("{problem}");
    }
    let checked = format!(
        "{} todos and {} label rule{}",
        todos.items.len(),
        rules.items.len(),
        if rules.items.len() == 1 { "" } else { "s" }
    );
    if problems.is_empty() {
        println!("{checked}, no problems");
        Ok(())
    } else {
        Err(format!(
            "{} problem{} in {checked}",
            problems.len(),
            if problems.len() == 1 { "" } else { "s" }
        ))
    }
}

pub const CLAIM_FLAGS: &[Flag] = &[
    option("label", Some('l')),
    switch("force", None),
    switch("no-commit", None),
];
const CLAIM_USAGE: &str = "todoer claim <name>... [--label l] [--force] [--no-commit]";

fn commit_todos(
    project: &Project,
    lock: &Lock,
    plan: &CommitPlan,
    action: &str,
    names: &[String],
) -> Result<(), String> {
    if plan.commit {
        let paths: Vec<String> = names.iter().map(|name| todo_path(name)).collect();
        project.commit(
            lock,
            &format!("todoer: {action} {}", names.join(", ")),
            &paths,
        )?;
    }
    report(plan);
    Ok(())
}

pub fn claim(args: &Args) -> Result<(), String> {
    let label = args.value("label");
    if args.positional.is_empty() && label.is_none() {
        return Err(format!("name a todo or a label - use {CLAIM_USAGE}"));
    }
    let project = Project::find()?;
    let me = project.me()?;
    let lock = project.lock()?;
    let todos = Todos::load(&project.root)?;
    let mut names: Vec<String> = Vec::new();
    for argument in &args.positional {
        let name = name_of(argument);
        todos.require(&name)?;
        if !names.contains(&name) {
            names.push(name);
        }
    }
    if let Some(label) = label {
        let labeled: Vec<&Todo> = todos
            .items
            .iter()
            .filter(|todo| todo.labels.iter().any(|entry| entry == label))
            .collect();
        if labeled.is_empty() {
            return Err(format!("no todo has the label {label} - run todoer labels"));
        }
        for todo in labeled {
            match todo.assignee.as_deref() {
                Some(assignee) if assignee != me => {
                    println!("skipped {}: claimed by {assignee}", todo.name);
                }
                _ if names.contains(&todo.name) => {}
                _ => names.push(todo.name.clone()),
            }
        }
    }
    let mut claimed = Vec::new();
    for name in &names {
        let todo = todos.require(name)?;
        match todo.assignee.as_deref() {
            Some(assignee) if assignee == me => {
                println!("{name} is already claimed by you");
                continue;
            }
            Some(assignee) => {
                return Err(format!(
                    "{name} is claimed by {assignee} - pick another todo, or ask {assignee} to run todoer unclaim"
                ));
            }
            None => {}
        }
        let waits: Vec<String> = todos
            .waits_for(todo)
            .into_iter()
            .filter(|other| !names.contains(other))
            .collect();
        if !waits.is_empty() && !args.has("force") {
            return Err(format!(
                "{name} waits for {} - claim a todo that is not blocked, claim it together with the todos that it waits for, or use --force",
                waits.join(", ")
            ));
        }
        claimed.push(todo.clone());
    }
    if claimed.is_empty() {
        return Ok(());
    }
    let paths: Vec<String> = claimed.iter().map(|todo| todo_path(&todo.name)).collect();
    let plan = project.plan_commit(args.has("no-commit"), &paths)?;
    let mut done = Vec::new();
    for mut todo in claimed {
        todo.set_assignee(Some(&me));
        todo.save(&project.root)?;
        println!("claimed {} for {me}", todo.name);
        done.push(todo.name);
    }
    commit_todos(&project, &lock, &plan, "claim", &done)
}

pub const UNCLAIM_FLAGS: &[Flag] = &[switch("no-commit", None)];

pub fn unclaim(args: &Args) -> Result<(), String> {
    if args.positional.is_empty() {
        return Err("use: todoer unclaim <name>... [--no-commit]".into());
    }
    let project = Project::find()?;
    let lock = project.lock()?;
    let mut todos = Vec::new();
    for argument in &args.positional {
        let name = name_of(argument);
        let todo = read_todo(&project.root, &name)?;
        if todo.assignee.is_none() {
            return Err(format!("{name} is not claimed"));
        }
        if !todos.iter().any(|other: &Todo| other.name == name) {
            todos.push(todo);
        }
    }
    let paths: Vec<String> = todos.iter().map(|todo| todo_path(&todo.name)).collect();
    let plan = project.plan_commit(args.has("no-commit"), &paths)?;
    let mut done = Vec::new();
    for mut todo in todos {
        let previous = todo.assignee.clone().unwrap_or_default();
        todo.set_assignee(None);
        todo.save(&project.root)?;
        println!("unclaimed {} (was {previous})", todo.name);
        done.push(todo.name);
    }
    commit_todos(&project, &lock, &plan, "unclaim", &done)
}
