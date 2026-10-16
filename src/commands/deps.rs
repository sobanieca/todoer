//! `dep add`, `dep rm` and `dep show`.

use crate::args::{Args, Flag, option};
use crate::colors;
use crate::commands::todos::state_color;
use crate::project::Project;
use crate::todos::{Todo, Todos, check_label, name_of, read_todo};
use std::collections::BTreeSet;

pub const FLAGS: &[Flag] = &[option("label", Some('l'))];

pub fn run(args: &Args) -> Result<(), String> {
    let usage = "todoer dep add|rm <name> [<name>...] [-l label]..., or todoer dep show <name>";
    let Some(action) = args.positional.first() else {
        return Err(format!("use: {usage}"));
    };
    let project = Project::find()?;
    let names: Vec<String> = args.positional[1..]
        .iter()
        .map(|argument| name_of(argument))
        .collect();
    let labels = args.values("label");
    match action.as_str() {
        "add" | "rm" if names.len() >= 2 || (names.len() == 1 && !labels.is_empty()) => {
            change(&project, &names, &labels, action == "add")
        }
        "show" if names.len() == 1 && labels.is_empty() => show(&project, &names[0]),
        _ => Err(format!("use: {usage}")),
    }
}

fn change(project: &Project, names: &[String], labels: &[String], add: bool) -> Result<(), String> {
    let root = &project.root;
    let _lock = project.lock()?;
    let mut todos = Todos::load(root)?;
    let first = &names[0];
    todos.require(first)?;
    let mut todo = read_todo(root, first)?;
    for other in &names[1..] {
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
                todos = todos.with(todo.clone());
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
    for label in labels {
        if add {
            check_label(label)?;
            let others = todos.labeled_except(label, first);
            if others.is_empty() {
                return Err(format!(
                    "no other todo has the label {label} - run todoer labels"
                ));
            }
            if let Some(other) = others.iter().find(|other| todos.reaches(other, first)) {
                return Err(format!(
                    "{first} cannot depend on the label {label} - {other} has the label and already depends on {first}, so this makes a cycle"
                ));
            }
            if !todo.depends_on_labels.contains(label) {
                let mut deps = todo.depends_on_labels.clone();
                deps.push(label.clone());
                todo.set_depends_on_labels(deps);
                todos = todos.with(todo.clone());
            }
            println!("{first} depends on the label {label}");
        } else {
            if !todo.depends_on_labels.contains(label) {
                return Err(format!("{first} does not depend on the label {label}"));
            }
            let deps = todo
                .depends_on_labels
                .iter()
                .filter(|dep| *dep != label)
                .cloned()
                .collect();
            todo.set_depends_on_labels(deps);
            println!("{first} no longer depends on the label {label}");
        }
    }
    todo.save(root)
}

fn label(todos: &Todos, name: &str) -> String {
    match todos.get(name) {
        Some(todo) => {
            let state = todos.state(todo);
            format!("{name}  {}", state_color(state, &format!("({state})")))
        }
        None => format!("{name}  {}", colors::red("(missing)")),
    }
}

fn show(project: &Project, name: &str) -> Result<(), String> {
    let todos = Todos::load(&project.root)?;
    todos.print_warnings();
    let todo = todos.require(name)?;
    println!("{}", label(&todos, name));
    println!();
    println!("waits for:");
    let mut seen = BTreeSet::new();
    seen.insert(name.to_string());
    if todo.depends_on.is_empty() && todo.depends_on_labels.is_empty() {
        println!("  nothing");
    }
    waits_tree(&todos, todo, 1, &mut seen);
    println!();
    println!("needed by:");
    let mut seen = BTreeSet::new();
    seen.insert(name.to_string());
    if todos.needed_by(name).is_empty() {
        println!("  nothing");
    }
    tree(&todos, todo, 1, &mut seen);
    Ok(())
}

fn waits_tree(todos: &Todos, todo: &Todo, depth: usize, seen: &mut BTreeSet<String>) {
    for name in &todo.depends_on {
        node(todos, name, depth, seen);
    }
    for label in &todo.depends_on_labels {
        let indent = "  ".repeat(depth);
        let members = todos.labeled_except(label, &todo.name);
        if members.is_empty() {
            println!("{indent}label {label}  {}", colors::red("(no todo)"));
        } else {
            println!("{indent}label {label}");
        }
        for name in &members {
            node(todos, name, depth + 1, seen);
        }
    }
}

fn node(todos: &Todos, name: &str, depth: usize, seen: &mut BTreeSet<String>) {
    let indent = "  ".repeat(depth);
    if !seen.insert(name.to_string()) {
        println!("{indent}{name}  {}", colors::gray("see above"));
        return;
    }
    println!("{indent}{}", label(todos, name));
    if let Some(child) = todos.get(name) {
        waits_tree(todos, child, depth + 1, seen);
    }
}

fn tree(todos: &Todos, todo: &Todo, depth: usize, seen: &mut BTreeSet<String>) {
    for name in todos.needed_by(&todo.name) {
        let indent = "  ".repeat(depth);
        if !seen.insert(name.clone()) {
            println!("{indent}{name}  {}", colors::gray("see above"));
            continue;
        }
        println!("{indent}{}", label(todos, &name));
        if let Some(child) = todos.get(&name) {
            tree(todos, child, depth + 1, seen);
        }
    }
}
