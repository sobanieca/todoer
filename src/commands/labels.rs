//! `label add`, `label rm`, `labels` and `progress`.

use crate::args::{Args, Flag, switch};
use crate::colors;
use crate::json::{self, Value};
use crate::project::Project;
use crate::prs;
use crate::todos::{Todo, Todos, check_label, read_todo};
use std::collections::BTreeMap;

pub fn label(args: &Args) -> Result<(), String> {
    let usage = "todoer label add|rm <path> <label>";
    args.expect_positional(3, 3, usage)?;
    let project = Project::find()?;
    let path = project.relative(&args.positional[1])?;
    let name = args.positional[2].as_str();
    let mut todo = read_todo(&project.current, &path)?;
    let mut labels = todo.labels.clone();
    match args.positional[0].as_str() {
        "add" => {
            check_label(name)?;
            if labels.iter().any(|label| label == name) {
                println!("{path} has the label {name}");
                return Ok(());
            }
            labels.push(name.to_string());
            println!("{path}: added the label {name}");
        }
        "rm" => {
            if !labels.iter().any(|label| label == name) {
                return Err(format!("{path} has no label {name}"));
            }
            labels.retain(|label| label != name);
            println!("{path}: removed the label {name}");
        }
        _ => return Err(format!("use: {usage}")),
    }
    todo.set_labels(labels);
    todo.save(&project.current)
}

pub const LABELS_FLAGS: &[Flag] = &[switch("json", None)];

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
    args.expect_positional(0, 0, "todoer labels [--json]")?;
    let project = Project::find()?;
    let todos = Todos::load(&project.current)?;
    todos.print_warnings();
    let map = label_counts(&todos);
    if args.has("json") {
        let items = map
            .iter()
            .map(|(label, items)| {
                Value::object(vec![
                    ("label", Value::str(label)),
                    ("todos", Value::Number(items.len() as f64)),
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
    }
    Ok(())
}

pub const PROGRESS_FLAGS: &[Flag] = &[switch("json", None)];

struct Progress<'a> {
    label: String,
    done: usize,
    total: usize,
    in_progress: Vec<&'a Todo>,
    ready: Vec<&'a Todo>,
    blocked: Vec<&'a Todo>,
}

fn progress_of<'a>(todos: &'a Todos, label: &str, items: &[&'a Todo]) -> Progress<'a> {
    let counted: Vec<&Todo> = items
        .iter()
        .copied()
        .filter(|todo| todo.status != "dropped")
        .collect();
    let paths: Vec<String> = counted
        .iter()
        .filter(|todo| !todo.is_closed())
        .map(|todo| todo.path.clone())
        .collect();
    let mut progress = Progress {
        label: label.to_string(),
        done: counted.iter().filter(|todo| todo.status == "done").count(),
        total: counted.len(),
        in_progress: Vec::new(),
        ready: Vec::new(),
        blocked: Vec::new(),
    };
    for path in todos.ordered(&paths) {
        let Some(todo) = todos.get(&path) else {
            continue;
        };
        match todos.state(todo) {
            "in-progress" => progress.in_progress.push(todo),
            "ready" => progress.ready.push(todo),
            _ => progress.blocked.push(todo),
        }
    }
    progress
}

fn percent(progress: &Progress) -> usize {
    (progress.done * 100)
        .checked_div(progress.total)
        .unwrap_or(0)
}

fn headline(progress: &Progress, width: usize) -> String {
    let bar_width = progress.total.clamp(1, 20);
    let filled = (progress.done * bar_width)
        .checked_div(progress.total)
        .unwrap_or(0);
    format!(
        "{}  {}/{} done ({}%)  [{}{}]",
        colors::pad(&colors::bold(&progress.label), width),
        progress.done,
        progress.total,
        percent(progress),
        colors::green(&"#".repeat(filled)),
        "-".repeat(bar_width - filled)
    )
}

fn pr_note(prs: &[prs::Pr], project: &Project, todo: &Todo) -> Option<String> {
    let pr = prs
        .iter()
        .find(|pr| pr.todos.iter().any(|path| path == &todo.path))?;
    let verdict = pr.verdict(project).replace('-', " ");
    Some(format!("pr: {}, {verdict}", pr.slug))
}

pub fn progress(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 1, "todoer progress [label] [--json]")?;
    let project = Project::find()?;
    let todos = Todos::load(&project.current)?;
    todos.print_warnings();
    let map = label_counts(&todos);
    let Some(label) = args.positional.first() else {
        let all: Vec<Progress> = map
            .iter()
            .map(|(label, items)| progress_of(&todos, label, items))
            .collect();
        if args.has("json") {
            let items = all
                .iter()
                .map(|progress| {
                    Value::object(vec![
                        ("label", Value::str(&progress.label)),
                        ("done", Value::Number(progress.done as f64)),
                        ("total", Value::Number(progress.total as f64)),
                        ("percent", Value::Number(percent(progress) as f64)),
                    ])
                })
                .collect();
            println!("{}", json::stringify(&Value::Array(items)));
            return Ok(());
        }
        if all.is_empty() {
            println!("no labels - add one with todoer label add <path> <label>");
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
    let progress = progress_of(&todos, label, items);
    let prs = prs::list(&project).unwrap_or_default();
    let outside = |path: &String| {
        todos
            .get(path)
            .is_none_or(|todo| !todo.labels.iter().any(|entry| entry == label))
    };
    if args.has("json") {
        let paths = |list: &[&Todo]| {
            Value::Array(
                list.iter()
                    .map(|todo| {
                        Value::object(vec![
                            ("path", Value::str(&todo.path)),
                            ("title", Value::str(&todo.title)),
                            ("assignee", Value::opt(todo.assignee.as_deref())),
                            ("pr", Value::opt(pr_note(&prs, &project, todo).as_deref())),
                            ("waitsFor", Value::strings(&todos.waits_for(todo))),
                        ])
                    })
                    .collect(),
            )
        };
        let value = Value::object(vec![
            ("label", Value::str(label)),
            ("done", Value::Number(progress.done as f64)),
            ("total", Value::Number(progress.total as f64)),
            ("percent", Value::Number(percent(&progress) as f64)),
            ("inProgress", paths(&progress.in_progress)),
            ("ready", paths(&progress.ready)),
            ("blocked", paths(&progress.blocked)),
        ]);
        println!("{}", json::stringify(&value));
        return Ok(());
    }
    println!("{}", headline(&progress, 0));
    let width = progress
        .in_progress
        .iter()
        .chain(&progress.ready)
        .chain(&progress.blocked)
        .map(|todo| todo.path.len())
        .max()
        .unwrap_or(0);
    let sections = [
        ("in progress", &progress.in_progress),
        ("ready", &progress.ready),
        ("blocked", &progress.blocked),
    ];
    for (name, list) in sections {
        if list.is_empty() {
            continue;
        }
        println!();
        println!("{name} ({})", list.len());
        for todo in list.iter() {
            let mut notes = Vec::new();
            if name == "blocked" {
                for path in todos.waits_for(todo) {
                    let suffix = if outside(&path) {
                        "  (outside label)"
                    } else {
                        ""
                    };
                    notes.push(format!("waits for {path}{suffix}"));
                }
            } else if let Some(assignee) = &todo.assignee {
                notes.push(assignee.clone());
            }
            if let Some(note) = pr_note(&prs, &project, todo) {
                notes.push(note);
            }
            let line = format!(
                "  {}  {}",
                colors::pad(&todo.path, width),
                colors::gray(&notes.join("  "))
            );
            println!("{}", line.trim_end());
        }
    }
    Ok(())
}
