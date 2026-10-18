//! `label add`, `label rm`, `labels` with its rules, and `progress`.

use crate::args::{Args, Flag, switch};
use crate::colors;
use crate::commands::todos::print_removed_labels;
use crate::json::{self, Value};
use crate::project::Project;
use crate::prs;
use crate::rules::{Rules, remove_rule, rename_rule, rule_file, rule_path, write_rule};
use crate::terminal;
use crate::todos::{
    Todo, Todos, check_label, name_of, read_todo, remove_label_dependencies_without_todos,
    valid_label,
};
use std::collections::BTreeMap;

pub fn label(args: &Args) -> Result<(), String> {
    let usage = "todoer label add|rm <name> <label>";
    args.expect_positional(3, 3, usage)?;
    let project = Project::find()?;
    let todo_name = name_of(&args.positional[1]);
    let name = args.positional[2].as_str();
    let _lock = project.lock()?;
    let mut todo = read_todo(&project.root, &todo_name)?;
    let mut labels = todo.labels.clone();
    let add = args.positional[0] == "add";
    match args.positional[0].as_str() {
        "add" => {
            check_label(name)?;
            if labels.iter().any(|label| label == name) {
                println!("{todo_name} has the label {name}");
                return Ok(());
            }
            labels.push(name.to_string());
            println!("{todo_name}: added the label {name}");
        }
        "rm" => {
            if !labels.iter().any(|label| label == name) {
                return Err(format!("{todo_name} has no label {name}"));
            }
            labels.retain(|label| label != name);
            println!("{todo_name}: removed the label {name}");
        }
        _ => return Err(format!("use: {usage}")),
    }
    todo.set_labels(labels);
    let todos = Todos::load(&project.root)?.with(todo.clone());
    if add {
        todos.check_no_cycle(std::slice::from_ref(&todo_name))?;
    }
    todo.save(&project.root)?;
    if !add {
        print_removed_labels(&remove_label_dependencies_without_todos(
            &project.root,
            &todos,
        )?);
    }
    Ok(())
}

pub const LABELS_FLAGS: &[Flag] = &[switch("json", None), switch("rm", None)];

fn label_counts(todos: &Todos) -> BTreeMap<String, Vec<&Todo>> {
    let mut map: BTreeMap<String, Vec<&Todo>> = BTreeMap::new();
    for todo in &todos.items {
        for label in &todo.labels {
            map.entry(label.clone()).or_default().push(todo);
        }
    }
    map
}

pub fn labels(args: &Args) -> Result<(), String> {
    match args.positional.first().map(String::as_str) {
        Some("rm") => return remove_label(args),
        Some("mv") => return move_label(args),
        Some("rule") => return rule(args),
        _ => {}
    }
    let usage = "todoer labels [--json]";
    args.expect_positional(0, 0, usage)?;
    if args.has("rm") {
        return Err(format!("use: {usage}"));
    }
    let project = Project::find()?;
    let todos = Todos::load(&project.root)?;
    todos.print_warnings();
    let rules = Rules::load(&project.root)?;
    rules.print_warnings();
    let mut map = label_counts(&todos);
    for label in rules.items.keys() {
        map.entry(label.clone()).or_default();
    }
    if args.has("json") {
        let items = map
            .iter()
            .map(|(label, items)| {
                Value::object(vec![
                    ("label", Value::str(label)),
                    ("todos", Value::Number(items.len() as f64)),
                    ("rule", Value::opt(rules.text(label))),
                ])
            })
            .collect();
        println!("{}", json::stringify(&Value::Array(items)));
        return Ok(());
    }
    if map.is_empty() {
        println!("no labels");
    }
    let width = map.keys().map(String::len).max().unwrap_or(0);
    for (label, items) in &map {
        let count = items.len();
        println!(
            "{}  {count} todo{}",
            colors::pad(label, width),
            if count == 1 { "" } else { "s" }
        );
        for line in rules.text(label).unwrap_or_default().lines() {
            println!("{}", format!("  {line}").trim_end());
        }
    }
    Ok(())
}

fn rule(args: &Args) -> Result<(), String> {
    let usage = "todoer labels rule <label> [text] | todoer labels rule <label> --rm";
    if args.positional.len() < 2 || args.has("json") {
        return Err(format!("use: {usage}"));
    }
    let label = args.positional[1].as_str();
    check_label(label)?;
    let project = Project::find()?;
    if args.has("rm") {
        args.expect_positional(2, 2, usage)?;
        let _lock = project.lock()?;
        if !rule_file(&project.root, label).is_file() {
            return Err(format!("the label {label} has no rule - run todoer labels"));
        }
        remove_rule(&project.root, label)?;
        println!("removed the rule of {label}");
        return Ok(());
    }
    let text = if args.positional.len() > 2 {
        args.positional[2..].join(" ")
    } else {
        terminal::read_stdin_if_piped()?
    };
    if text.trim().is_empty() {
        return Err(format!(
            "the rule of {label} has no text - give the text, or pipe it on stdin"
        ));
    }
    let _lock = project.lock()?;
    write_rule(&project.root, label, &text)?;
    println!("{}", rule_path(label));
    Ok(())
}

fn todos_with_label(project: &Project, label: &str) -> Result<Vec<String>, String> {
    let todos = Todos::load(&project.root)?;
    todos.print_warnings();
    Ok(todos
        .items
        .iter()
        .filter(|todo| todo.labels.iter().any(|other| other == label))
        .map(|todo| todo.name.clone())
        .collect())
}

fn no_label(label: &str) -> String {
    format!("no todo and no rule has the label {label} - run todoer labels")
}

fn remove_label(args: &Args) -> Result<(), String> {
    let usage = "todoer labels rm <label>";
    args.expect_positional(2, 2, usage)?;
    if args.has("json") || args.has("rm") {
        return Err(format!("use: {usage}"));
    }
    let label = args.positional[1].as_str();
    let project = Project::find()?;
    let _lock = project.lock()?;
    let names = todos_with_label(&project, label)?;
    let has_rule = rule_file(&project.root, label).is_file();
    if names.is_empty() && !has_rule {
        return Err(no_label(label));
    }
    for name in names {
        let mut todo = read_todo(&project.root, &name)?;
        let labels = todo
            .labels
            .iter()
            .filter(|other| *other != label)
            .cloned()
            .collect();
        todo.set_labels(labels);
        todo.save(&project.root)?;
        println!("{name}: removed the label {label}");
    }
    print_removed_labels(&remove_label_dependencies_without_todos(
        &project.root,
        &Todos::load(&project.root)?,
    )?);
    if has_rule {
        remove_rule(&project.root, label)?;
        println!("removed the rule of {label}");
    }
    Ok(())
}

fn move_label(args: &Args) -> Result<(), String> {
    let usage = "todoer labels mv <old> <new>";
    args.expect_positional(3, 3, usage)?;
    if args.has("json") || args.has("rm") {
        return Err(format!("use: {usage}"));
    }
    let old = args.positional[1].as_str();
    let new = args.positional[2].as_str();
    check_label(new)?;
    if old == new {
        return Err(format!(
            "{old} and {new} are the same label - give a new name"
        ));
    }
    let project = Project::find()?;
    let _lock = project.lock()?;
    let names = todos_with_label(&project, old)?;
    // A bad label in a todo can be renamed, but only a valid label has a rule file.
    let has_rule = valid_label(old) && rule_file(&project.root, old).is_file();
    if names.is_empty() && !has_rule {
        return Err(no_label(old));
    }
    if has_rule && rule_file(&project.root, new).is_file() {
        return Err(format!(
            "{old} and {new} both have a rule - remove one rule first: todoer labels rule <label> --rm"
        ));
    }
    let mut todos = Todos::load(&project.root)?;
    let mut changed = Vec::new();
    for todo in &todos.items {
        let has_old = todo.has_label(old);
        let depends_on_old = todo.depends_on_labels.iter().any(|label| label == old);
        if !has_old && !depends_on_old {
            continue;
        }
        let mut todo = todo.clone();
        todo.set_labels(renamed(&todo.labels, old, new));
        todo.set_depends_on_labels(renamed(&todo.depends_on_labels, old, new));
        changed.push((todo, has_old, depends_on_old));
    }
    for (todo, _, _) in &changed {
        todos = todos.with(todo.clone());
    }
    let names: Vec<String> = changed
        .iter()
        .map(|(todo, _, _)| todo.name.clone())
        .collect();
    todos.check_no_cycle(&names)?;
    for (todo, has_old, depends_on_old) in &changed {
        todo.save(&project.root)?;
        if *has_old {
            println!("{}: renamed the label {old} -> {new}", todo.name);
        }
        if *depends_on_old {
            println!("{}: renamed {old} -> {new} in dependsOnLabels", todo.name);
        }
    }
    if has_rule {
        rename_rule(&project.root, old, new)?;
        println!("renamed the rule {old} -> {new}");
    }
    Ok(())
}

fn renamed(labels: &[String], old: &str, new: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for label in labels {
        let label = if label == old { new } else { label.as_str() };
        if !out.iter().any(|other| other == label) {
            out.push(label.to_string());
        }
    }
    out
}

pub const PROGRESS_FLAGS: &[Flag] = &[switch("json", None)];

struct Progress<'a> {
    label: String,
    total: usize,
    in_progress: Vec<&'a Todo>,
    in_pr: Vec<&'a Todo>,
    not_started: Vec<&'a Todo>,
}

fn progress_of<'a>(
    todos: &'a Todos,
    open: &[prs::Pr],
    label: &str,
    items: &[&'a Todo],
) -> Progress<'a> {
    let names: Vec<String> = items.iter().map(|todo| todo.name.clone()).collect();
    let mut progress = Progress {
        label: label.to_string(),
        total: items.len(),
        in_progress: Vec::new(),
        in_pr: Vec::new(),
        not_started: Vec::new(),
    };
    for name in todos.ordered(&names) {
        let Some(todo) = todos.get(&name) else {
            continue;
        };
        if !prs::of_todo(open, &todo.name).is_empty() {
            progress.in_pr.push(todo);
        } else if todos.state(todo) == "in-progress" {
            progress.in_progress.push(todo);
        } else {
            progress.not_started.push(todo);
        }
    }
    progress
}

fn headline(progress: &Progress, width: usize) -> String {
    let bar_width = progress.total.clamp(1, 20);
    let part = |count: usize| (count * bar_width).checked_div(progress.total).unwrap_or(0);
    let in_pr = part(progress.in_pr.len());
    let in_progress = part(progress.in_progress.len());
    format!(
        "{}  {} in progress, {} in a pull request, {} not started  [{}{}{}]",
        colors::pad(&colors::bold(&progress.label), width),
        progress.in_progress.len(),
        progress.in_pr.len(),
        progress.not_started.len(),
        colors::green(&"#".repeat(in_pr)),
        colors::yellow(&"=".repeat(in_progress)),
        "-".repeat(bar_width - in_pr - in_progress)
    )
}

pub fn progress(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 1, "todoer progress [label] [--json]")?;
    let project = Project::find()?;
    let todos = Todos::load(&project.root)?;
    todos.print_warnings();
    let open = prs::list(&project)?;
    let map = label_counts(&todos);
    let Some(label) = args.positional.first() else {
        let all: Vec<Progress> = map
            .iter()
            .map(|(label, items)| progress_of(&todos, &open, label, items))
            .collect();
        if args.has("json") {
            let items = all
                .iter()
                .map(|progress| {
                    Value::object(vec![
                        ("label", Value::str(&progress.label)),
                        ("total", Value::Number(progress.total as f64)),
                        (
                            "inProgress",
                            Value::Number(progress.in_progress.len() as f64),
                        ),
                        ("inPullRequest", Value::Number(progress.in_pr.len() as f64)),
                        (
                            "notStarted",
                            Value::Number(progress.not_started.len() as f64),
                        ),
                    ])
                })
                .collect();
            println!("{}", json::stringify(&Value::Array(items)));
            return Ok(());
        }
        if all.is_empty() {
            println!("no labels - add one with todoer label add <name> <label>");
        }
        let width = map.keys().map(String::len).max().unwrap_or(0);
        for progress in &all {
            println!("{}", headline(progress, width));
        }
        return Ok(());
    };
    let items = map
        .get(label)
        .ok_or_else(|| format!("no todo has the label {label} - run todoer labels"))?;
    let progress = progress_of(&todos, &open, label, items);
    let outside = |name: &String| {
        todos
            .get(name)
            .is_none_or(|todo| !todo.labels.iter().any(|entry| entry == label))
    };
    if args.has("json") {
        let list_json = |list: &[&Todo]| {
            Value::Array(
                list.iter()
                    .map(|todo| {
                        let names: Vec<String> = prs::of_todo(&open, &todo.name)
                            .iter()
                            .map(|pr| pr.name.clone())
                            .collect();
                        Value::object(vec![
                            ("name", Value::str(&todo.name)),
                            ("state", Value::str(todos.state(todo))),
                            ("assignee", Value::opt(todo.assignee.as_deref())),
                            ("prs", Value::strings(&names)),
                            ("waitsFor", Value::strings(&todos.waits_for(todo))),
                        ])
                    })
                    .collect(),
            )
        };
        let value = Value::object(vec![
            ("label", Value::str(label)),
            ("total", Value::Number(progress.total as f64)),
            ("inProgress", list_json(&progress.in_progress)),
            ("inPullRequest", list_json(&progress.in_pr)),
            ("notStarted", list_json(&progress.not_started)),
        ]);
        println!("{}", json::stringify(&value));
        return Ok(());
    }
    println!("{}", headline(&progress, 0));
    let width = progress
        .in_progress
        .iter()
        .chain(&progress.in_pr)
        .chain(&progress.not_started)
        .map(|todo| todo.name.len())
        .max()
        .unwrap_or(0);
    let sections = [
        ("in progress", &progress.in_progress),
        ("in a pull request", &progress.in_pr),
        ("not started", &progress.not_started),
    ];
    for (name, list) in sections {
        if list.is_empty() {
            continue;
        }
        println!();
        println!("{name} ({})", list.len());
        for todo in list.iter() {
            let mut notes = Vec::new();
            match todos.state(todo) {
                "blocked" => {
                    for name in todos.waits_for(todo) {
                        let suffix = if outside(&name) {
                            "  (outside label)"
                        } else {
                            ""
                        };
                        notes.push(format!("waits for {name}{suffix}"));
                    }
                }
                "available" => notes.push("available".to_string()),
                _ => notes.extend(todo.assignee.clone()),
            }
            notes.extend(prs::note(&prs::of_todo(&open, &todo.name)));
            let line = format!(
                "  {}  {}",
                colors::pad(&todo.name, width),
                colors::gray(&notes.join("  "))
            );
            println!("{}", line.trim_end());
        }
    }
    Ok(())
}
