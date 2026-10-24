//! `ls`, `show`, `add` and `done`. No command commits.

use crate::args::{Args, Flag, switch};
use crate::colors;
use crate::git;
use crate::json::{self, Value};
use crate::project::{Project, write_file};
use crate::prs::{self, Pr};
use crate::terminal;
use crate::todos::{
    Todo, Todos, check_name, name_of, remove_todo, todo_file, todo_path, todos_dir,
};
use std::path::{Path, PathBuf};

pub const LS_FLAGS: &[Flag] = &[switch("json", None)];

fn print_rows(todos: &Todos, prs: &[Pr], indent: &str) {
    let width = todos
        .items
        .iter()
        .map(|todo| todo.name.len())
        .max()
        .unwrap_or(0);
    for todo in &todos.items {
        let notes = prs::note(&prs::of_todo(prs, &todo.name)).unwrap_or_default();
        let line = format!(
            "{indent}{}  {}",
            colors::pad(&todo.name, width),
            colors::gray(&notes)
        );
        println!("{}", line.trim_end());
    }
}

pub fn todo_json(prs: &[Pr], todo: &Todo) -> Vec<(&'static str, Value)> {
    let names: Vec<String> = prs::of_todo(prs, &todo.name)
        .iter()
        .map(|pr| pr.name.clone())
        .collect();
    vec![
        ("name", Value::str(&todo.name)),
        ("prs", Value::strings(&names)),
    ]
}

pub fn ls(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 0, "todoer ls [--json]")?;
    let cwd = std::env::current_dir()
        .map_err(|error| format!("cannot read the current folder: {error}"))?;
    if git::toplevel(&cwd).is_none() {
        return ls_projects(args, &cwd);
    }
    let project = Project::find_from(&cwd)?;
    let todos = Todos::load(&project.root)?;
    todos.print_warnings();
    let open = prs::list(&project)?;
    if args.has("json") {
        let items = todos
            .items
            .iter()
            .map(|todo| Value::object(todo_json(&open, todo)))
            .collect();
        println!("{}", json::stringify(&Value::Array(items)));
        return Ok(());
    }
    if todos.items.is_empty() {
        println!("no todos");
        return Ok(());
    }
    print_rows(&todos, &open, "");
    Ok(())
}

pub const SHOW_FLAGS: &[Flag] = &[switch("json", None)];

pub fn show(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, "todoer show <name> [--json]")?;
    let project = Project::find()?;
    let todos = Todos::load(&project.root)?;
    todos.print_warnings();
    let name = name_of(&args.positional[0]);
    let todo = todos.require(&name)?;
    let open = prs::list(&project)?;
    let text = todo.text.trim();
    if args.has("json") {
        let mut fields = todo_json(&open, todo);
        fields.push(("file", Value::str(&todo_path(&name))));
        fields.push(("text", Value::str(text)));
        println!("{}", json::stringify(&Value::object(fields)));
        return Ok(());
    }
    let rows = [
        ("file", vec![todo_path(&name)]),
        ("pr", prs::names_with_comments(&prs::of_todo(&open, &name))),
    ];
    println!("{}", colors::bold(&name));
    for (key, values) in rows {
        let value = if values.is_empty() {
            colors::gray("none")
        } else {
            values.join(", ")
        };
        println!("  {}  {value}", colors::pad(&format!("{key}:"), 5));
    }
    if !text.is_empty() {
        println!();
        println!("{text}");
    }
    Ok(())
}

fn projects_in(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect_projects(folder, &mut found);
    found.sort();
    found
}

fn collect_projects(folder: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let is_folder = entry.file_type().is_ok_and(|kind| kind.is_dir());
        let hidden = entry.file_name().to_string_lossy().starts_with('.');
        if !is_folder || hidden {
            continue;
        }
        let path = entry.path();
        if path.join(".git").is_dir() {
            if todos_dir(&path).is_dir() {
                found.push(path);
            }
        } else if !path.join(".git").exists() {
            collect_projects(&path, found);
        }
    }
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
            .strip_prefix(folder)
            .unwrap_or(root)
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        let project = Project::find_from(root)?;
        let branch = project.branch_name()?;
        let open = prs::list(&project)?;
        let todos = Todos::load(root)?;
        for problem in todos.problems() {
            crate::logger::warn(&format!("{name}: {problem}"));
        }
        if args.has("json") {
            for todo in &todos.items {
                let mut fields = vec![
                    ("project", Value::str(&name)),
                    ("branch", Value::str(&branch)),
                ];
                fields.extend(todo_json(&open, todo));
                items.push(Value::object(fields));
            }
            continue;
        }
        if todos.items.is_empty() {
            continue;
        }
        if printed {
            println!();
        }
        println!("{}  {}", colors::bold(&name), colors::gray(&branch));
        print_rows(&todos, &open, "  ");
        printed = true;
    }
    if args.has("json") {
        println!("{}", json::stringify(&Value::Array(items)));
    } else if !printed {
        println!("no todos");
    }
    Ok(())
}

pub fn add(args: &Args) -> Result<(), String> {
    let usage = "todoer add <name> [text]";
    let Some(name) = args.positional.first() else {
        return Err(format!("a todo needs a name - use {usage}"));
    };
    check_name(name)?;
    let project = Project::find()?;
    let _lock = project.lock()?;
    let todos = Todos::load(&project.root)?;
    if todos.get(name).is_some() || todo_file(&project.root, name).exists() {
        return Err(format!("the todo {name} exists - pick another name"));
    }
    let text = if args.positional.len() > 1 {
        args.positional[1..].join(" ")
    } else {
        terminal::read_stdin_if_piped()?
    };
    let text = text.trim();
    let content = if text.is_empty() {
        String::new()
    } else {
        format!("{text}\n")
    };
    write_file(&todo_file(&project.root, name), &content)?;
    println!("{}", todo_path(name));
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
    Ok(())
}
