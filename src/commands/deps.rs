//! `dep add`, `dep rm` and `dep show`.

use crate::args::Args;
use crate::colors;
use crate::commands::todos::state_color;
use crate::project::Project;
use crate::todos::{Todo, Todos, read_todo};
use std::collections::BTreeSet;

pub fn run(args: &Args) -> Result<(), String> {
    let usage = "todoer dep add|rm <path> <path> [...], or todoer dep show <path>";
    let Some(action) = args.positional.first() else {
        return Err(format!("use: {usage}"));
    };
    let project = Project::find()?;
    let paths = args.positional[1..]
        .iter()
        .map(|argument| project.relative(argument))
        .collect::<Result<Vec<_>, _>>()?;
    match action.as_str() {
        "add" | "rm" if paths.len() >= 2 => change(&project, &paths, action == "add"),
        "show" if paths.len() == 1 => show(&project, &paths[0]),
        _ => Err(format!("use: {usage}")),
    }
}

fn change(project: &Project, paths: &[String], add: bool) -> Result<(), String> {
    let root = &project.current;
    let mut todos = Todos::load(root)?;
    let first = &paths[0];
    todos.require(first)?;
    let mut todo = read_todo(root, first)?;
    for other in &paths[1..] {
        if add {
            todos.require(other)?;
            if other == first {
                return Err(format!("{first} cannot depend on itself"));
            }
            if todos.reaches(other, first) {
                return Err(format!(
                    "{first} cannot depend on {other} - {other} already depends on {first}, so this makes a cycle"
                ));
            }
            if !todo.depends_on.contains(other) {
                let mut deps = todo.depends_on.clone();
                deps.push(other.clone());
                todo.set_depends_on(deps);
                todos = Todos::from_items(
                    todos
                        .items
                        .into_iter()
                        .map(|entry| {
                            if entry.path == *first {
                                todo.clone()
                            } else {
                                entry
                            }
                        })
                        .collect(),
                );
            }
            println!("{first} depends on {other}");
        } else {
            if !todo.depends_on.contains(other) {
                return Err(format!("{first} does not depend on {other}"));
            }
            let deps = todo
                .depends_on
                .iter()
                .filter(|dep| *dep != other)
                .cloned()
                .collect();
            todo.set_depends_on(deps);
            println!("{first} no longer depends on {other}");
        }
    }
    todo.save(root)
}

fn label(todos: &Todos, path: &str) -> String {
    match todos.get(path) {
        Some(todo) => {
            let state = todos.state(todo);
            format!(
                "{}  {}  {}",
                path,
                todo.title,
                state_color(state, &format!("({state})"))
            )
        }
        None => format!("{path}  {}", colors::red("(missing)")),
    }
}

fn show(project: &Project, path: &str) -> Result<(), String> {
    let todos = Todos::load(&project.current)?;
    todos.print_warnings();
    let todo = todos.require(path)?;
    println!("{}", label(&todos, path));
    println!();
    println!("waits for:");
    let mut seen = BTreeSet::new();
    seen.insert(path.to_string());
    if todo.depends_on.is_empty() {
        println!("  nothing");
    }
    tree(&todos, todo, 1, &mut seen, |todo| todo.depends_on.clone());
    println!();
    println!("needed by:");
    let mut seen = BTreeSet::new();
    seen.insert(path.to_string());
    if todos.needed_by(path).is_empty() {
        println!("  nothing");
    }
    tree(&todos, todo, 1, &mut seen, |todo| {
        todos.needed_by(&todo.path)
    });
    Ok(())
}

fn tree(
    todos: &Todos,
    todo: &Todo,
    depth: usize,
    seen: &mut BTreeSet<String>,
    next: impl Fn(&Todo) -> Vec<String> + Copy,
) {
    for path in next(todo) {
        let indent = "  ".repeat(depth);
        if !seen.insert(path.clone()) {
            println!("{indent}{path}  {}", colors::gray("see above"));
            continue;
        }
        println!("{indent}{}", label(todos, &path));
        if let Some(child) = todos.get(&path) {
            tree(todos, child, depth + 1, seen, next);
        }
    }
}
