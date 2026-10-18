//! The todos: the markdown files `.todoer/todos/<name>.md` of the current checkout. The name is the
//! id and the title of a todo.

use crate::frontmatter::Document;
use crate::project::{DATA_DIR, read_file, write_file};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Todo {
    pub name: String,
    pub doc: Document,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub depends_on: Vec<String>,
    pub depends_on_labels: Vec<String>,
}

impl Todo {
    pub fn from_document(name: &str, doc: Document) -> Todo {
        Todo {
            name: name.to_string(),
            assignee: doc.text("assignee"),
            labels: doc.list("labels"),
            depends_on: doc.list("dependsOn"),
            depends_on_labels: doc.list("dependsOnLabels"),
            doc,
        }
    }

    pub fn set_assignee(&mut self, assignee: Option<&str>) {
        self.assignee = assignee.map(str::to_string);
        match assignee {
            Some(email) => self.doc.set("assignee", email),
            None => self.doc.remove("assignee"),
        }
    }

    pub fn set_labels(&mut self, labels: Vec<String>) {
        self.doc.set_list("labels", &labels);
        self.labels = labels;
    }

    pub fn set_depends_on(&mut self, depends_on: Vec<String>) {
        self.doc.set_list("dependsOn", &depends_on);
        self.depends_on = depends_on;
    }

    pub fn set_depends_on_labels(&mut self, labels: Vec<String>) {
        self.doc.set_list("dependsOnLabels", &labels);
        self.depends_on_labels = labels;
    }

    pub fn has_label(&self, label: &str) -> bool {
        self.labels.iter().any(|entry| entry == label)
    }

    pub fn save(&self, root: &Path) -> Result<(), String> {
        write_file(&todo_file(root, &self.name), &self.doc.render())
    }
}

pub fn todos_dir(root: &Path) -> PathBuf {
    root.join(DATA_DIR).join("todos")
}

pub fn todo_file(root: &Path, name: &str) -> PathBuf {
    todos_dir(root).join(format!("{name}.md"))
}

pub fn todo_path(name: &str) -> String {
    format!("{DATA_DIR}/todos/{name}.md")
}

pub fn valid_name(text: &str) -> bool {
    let end = |c: Option<char>| c.is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    end(text.chars().next())
        && end(text.chars().last())
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn valid_label(label: &str) -> bool {
    valid_name(label)
}

pub fn check_label(label: &str) -> Result<(), String> {
    if valid_label(label) {
        Ok(())
    } else {
        Err(format!(
            "\"{label}\" is not a label - use lowercase letters, digits and hyphens, with a letter or digit at each end"
        ))
    }
}

pub fn check_name(name: &str) -> Result<(), String> {
    if valid_name(name) {
        return Ok(());
    }
    match slug(name, 60) {
        good if good.is_empty() => Err(format!(
            "\"{name}\" is not a todo name - use lowercase letters, digits and hyphens, with a letter or digit at each end"
        )),
        good => Err(format!("\"{name}\" is not a todo name - use {good}")),
    }
}

pub fn name_of(argument: &str) -> String {
    let last = argument
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(argument);
    last.strip_suffix(".md").unwrap_or(last).to_string()
}

pub fn read_todo(root: &Path, name: &str) -> Result<Todo, String> {
    let file = todo_file(root, name);
    if !valid_name(name) || !file.is_file() {
        return Err(format!("there is no todo {name} - run todoer ls"));
    }
    Ok(Todo::from_document(
        name,
        Document::parse(&read_file(&file)?),
    ))
}

pub fn remove_todo(root: &Path, name: &str) -> Result<(), String> {
    std::fs::remove_file(todo_file(root, name))
        .map_err(|error| format!("cannot remove {}: {error}", todo_path(name)))
}

pub struct Todos {
    pub items: Vec<Todo>,
    index: BTreeMap<String, usize>,
    by_label: BTreeMap<String, Vec<usize>>,
}

impl Todos {
    pub fn load(root: &Path) -> Result<Todos, String> {
        let Ok(entries) = std::fs::read_dir(todos_dir(root)) else {
            return Ok(Todos::from_items(Vec::new()));
        };
        let mut items = Vec::new();
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().into_owned();
            let Some(name) = file_name.strip_suffix(".md") else {
                continue;
            };
            if name.starts_with('.') || !path.is_file() {
                continue;
            }
            items.push(Todo::from_document(
                name,
                Document::parse(&read_file(&path)?),
            ));
        }
        Ok(Todos::from_items(items))
    }

    pub fn from_items(mut items: Vec<Todo>) -> Todos {
        items.sort_by(|a, b| a.name.cmp(&b.name));
        let index = items
            .iter()
            .enumerate()
            .map(|(at, todo)| (todo.name.clone(), at))
            .collect();
        let mut by_label: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (at, todo) in items.iter().enumerate() {
            for label in &todo.labels {
                let entries = by_label.entry(label.clone()).or_default();
                if !entries.contains(&at) {
                    entries.push(at);
                }
            }
        }
        Todos {
            items,
            index,
            by_label,
        }
    }

    pub fn with(self, todo: Todo) -> Todos {
        let mut items: Vec<Todo> = self
            .items
            .into_iter()
            .filter(|entry| entry.name != todo.name)
            .collect();
        items.push(todo);
        Todos::from_items(items)
    }

    pub fn get(&self, name: &str) -> Option<&Todo> {
        self.index.get(name).map(|at| &self.items[*at])
    }

    pub fn require(&self, name: &str) -> Result<&Todo, String> {
        self.get(name)
            .ok_or_else(|| format!("there is no todo {name} - run todoer ls"))
    }

    pub fn labeled_except(&self, label: &str, name: &str) -> Vec<String> {
        self.by_label
            .get(label)
            .map(|entries| {
                entries
                    .iter()
                    .map(|at| self.items[*at].name.clone())
                    .filter(|other| other != name)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn edges(&self, todo: &Todo) -> Vec<String> {
        let mut out = todo.depends_on.clone();
        for label in &todo.depends_on_labels {
            for name in self.labeled_except(label, &todo.name) {
                if !out.contains(&name) {
                    out.push(name);
                }
            }
        }
        out
    }

    pub fn waits_for(&self, todo: &Todo) -> Vec<String> {
        self.edges(todo)
            .into_iter()
            .filter(|name| *name != todo.name)
            .filter(|name| self.get(name).is_some())
            .collect()
    }

    pub fn waits_summary(&self, todo: &Todo) -> Vec<String> {
        let mut out: Vec<String> = todo
            .depends_on
            .iter()
            .filter(|name| **name != todo.name && self.get(name).is_some())
            .cloned()
            .collect();
        for label in &todo.depends_on_labels {
            if !self.labeled_except(label, &todo.name).is_empty() {
                out.push(format!("label {label}"));
            }
        }
        out
    }

    pub fn state(&self, todo: &Todo) -> &'static str {
        if todo.assignee.is_some() {
            "in-progress"
        } else if self.waits_for(todo).is_empty() {
            "available"
        } else {
            "blocked"
        }
    }

    pub fn needed_by(&self, name: &str) -> Vec<String> {
        self.items
            .iter()
            .filter(|todo| todo.name != name && self.edges(todo).iter().any(|dep| dep == name))
            .map(|todo| todo.name.clone())
            .collect()
    }

    pub fn reaches(&self, from: &str, to: &str) -> bool {
        self.path(from, to).is_some()
    }

    fn path(&self, from: &str, to: &str) -> Option<Vec<String>> {
        let mut seen = BTreeSet::new();
        let mut stack = vec![vec![from.to_string()]];
        while let Some(path) = stack.pop() {
            let name = path.last()?;
            if name == to {
                return Some(path);
            }
            if !seen.insert(name.clone()) {
                continue;
            }
            if let Some(todo) = self.get(name) {
                for next in self.edges(todo).into_iter().rev() {
                    let mut longer = path.clone();
                    longer.push(next);
                    stack.push(longer);
                }
            }
        }
        None
    }

    pub fn cycle_through(&self, name: &str) -> Option<Vec<String>> {
        let todo = self.get(name)?;
        self.edges(todo)
            .iter()
            .filter(|next| *next != name)
            .find_map(|next| self.path(next, name))
            .map(|path| {
                let mut cycle = vec![name.to_string()];
                cycle.extend(path);
                cycle
            })
    }

    pub fn check_no_cycle(&self, names: &[String]) -> Result<(), String> {
        for name in names {
            if let Some(cycle) = self.cycle_through(name) {
                return Err(format!(
                    "this makes the dependency cycle {} - remove a dependency first",
                    cycle.join(" -> ")
                ));
            }
        }
        Ok(())
    }

    pub fn cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut done = BTreeSet::new();
        for todo in &self.items {
            let mut stack = Vec::new();
            self.visit(&todo.name, &mut stack, &mut done, &mut cycles);
        }
        cycles
    }

    fn visit(
        &self,
        name: &str,
        stack: &mut Vec<String>,
        done: &mut BTreeSet<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if let Some(at) = stack.iter().position(|entry| entry == name) {
            let mut cycle = stack[at..].to_vec();
            cycle.push(name.to_string());
            cycles.push(cycle);
            return;
        }
        if done.contains(name) {
            return;
        }
        let Some(todo) = self.get(name) else {
            return;
        };
        stack.push(name.to_string());
        for dep in self.edges(todo) {
            self.visit(&dep, stack, done, cycles);
        }
        stack.pop();
        done.insert(name.to_string());
    }

    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        for todo in &self.items {
            if let Err(error) = check_name(&todo.name) {
                problems.push(format!("{}: {error}", todo.name));
            }
            for label in &todo.labels {
                if !valid_label(label) {
                    problems.push(format!(
                        "{}: bad label \"{label}\" - use lowercase letters, digits and hyphens, with a letter or digit at each end",
                        todo.name
                    ));
                }
            }
            if let Some(assignee) = &todo.assignee
                && !assignee.contains('@')
            {
                problems.push(format!(
                    "{}: bad assignee \"{assignee}\" - use a git email",
                    todo.name
                ));
            }
            for dep in &todo.depends_on {
                if dep == &todo.name {
                    problems.push(format!("{}: dependsOn names the todo itself", todo.name));
                } else if self.get(dep).is_none() {
                    problems.push(format!(
                        "{}: dependsOn names {dep}, which is not a todo",
                        todo.name
                    ));
                }
            }
            for label in &todo.depends_on_labels {
                if !valid_label(label) {
                    problems.push(format!(
                        "{}: bad label \"{label}\" in dependsOnLabels - use lowercase letters, digits and hyphens, with a letter or digit at each end",
                        todo.name
                    ));
                } else if self.labeled_except(label, &todo.name).is_empty() {
                    problems.push(format!(
                        "{}: dependsOnLabels names {label}, which no other todo has",
                        todo.name
                    ));
                }
            }
        }
        for cycle in self.cycles() {
            if cycle.len() > 2 {
                problems.push(format!(
                    "{}: dependency cycle {}",
                    cycle[0],
                    cycle.join(" -> ")
                ));
            }
        }
        problems
    }

    pub fn print_warnings(&self) {
        for problem in self.problems() {
            crate::logger::warn(&problem);
        }
    }

    pub fn ordered(&self, names: &[String]) -> Vec<String> {
        let wanted: BTreeSet<&String> = names.iter().collect();
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for name in names {
            self.order_visit(name, &wanted, &mut seen, &mut out);
        }
        out
    }

    fn order_visit(
        &self,
        name: &str,
        wanted: &BTreeSet<&String>,
        seen: &mut BTreeSet<String>,
        out: &mut Vec<String>,
    ) {
        if !seen.insert(name.to_string()) {
            return;
        }
        if let Some(todo) = self.get(name) {
            for dep in self.edges(todo) {
                if wanted.contains(&dep) {
                    self.order_visit(&dep, wanted, seen, out);
                }
            }
        }
        out.push(name.to_string());
    }
}

pub fn remove_dependency_on(root: &Path, todos: &Todos, name: &str) -> Result<Vec<String>, String> {
    let mut changed = Vec::new();
    for todo in &todos.items {
        if todo.name == name || !todo.depends_on.iter().any(|dep| dep == name) {
            continue;
        }
        let mut todo = todo.clone();
        let deps = todo
            .depends_on
            .iter()
            .filter(|dep| *dep != name)
            .cloned()
            .collect();
        todo.set_depends_on(deps);
        todo.save(root)?;
        changed.push(todo.name);
    }
    Ok(changed)
}

pub fn remove_label_dependencies_without_todos(
    root: &Path,
    todos: &Todos,
) -> Result<Vec<(String, String)>, String> {
    let mut removed = Vec::new();
    for todo in &todos.items {
        let (keep, gone): (Vec<String>, Vec<String>) = todo
            .depends_on_labels
            .iter()
            .cloned()
            .partition(|label| !todos.labeled_except(label, &todo.name).is_empty());
        if gone.is_empty() {
            continue;
        }
        let mut todo = todo.clone();
        todo.set_depends_on_labels(keep);
        todo.save(root)?;
        removed.extend(gone.into_iter().map(|label| (todo.name.clone(), label)));
    }
    Ok(removed)
}

pub fn slug(text: &str, max: usize) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    let mut out = out.trim_end_matches('-').to_string();
    if out.len() > max {
        out.truncate(max);
        if let Some(cut) = out.rfind('-').filter(|cut| *cut > max / 2) {
            out.truncate(cut);
        }
        out = out.trim_end_matches('-').to_string();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn todo(name: &str, assignee: Option<&str>, deps: &[&str]) -> Todo {
        let mut doc = Document::new();
        if let Some(email) = assignee {
            doc.set("assignee", email);
        }
        let deps: Vec<String> = deps.iter().map(|dep| dep.to_string()).collect();
        doc.set_list("dependsOn", &deps);
        Todo::from_document(name, doc)
    }

    #[test]
    fn finds_states() {
        let todos = Todos::from_items(vec![
            todo("a", None, &["b"]),
            todo("b", Some("me@x.io"), &[]),
            todo("c", None, &["gone"]),
            todo("d", Some("me@x.io"), &["a"]),
        ]);
        assert_eq!(todos.state(todos.get("a").unwrap()), "blocked");
        assert_eq!(todos.state(todos.get("b").unwrap()), "in-progress");
        assert_eq!(todos.state(todos.get("c").unwrap()), "available");
        assert_eq!(todos.state(todos.get("d").unwrap()), "in-progress");
        assert_eq!(todos.needed_by("b"), vec!["a"]);
    }

    fn labeled(name: &str, assignee: Option<&str>, labels: &[&str], on_labels: &[&str]) -> Todo {
        let mut todo = todo(name, assignee, &[]);
        todo.set_labels(labels.iter().map(|label| label.to_string()).collect());
        todo.set_depends_on_labels(on_labels.iter().map(|label| label.to_string()).collect());
        todo
    }

    #[test]
    fn waits_for_the_todos_of_a_label() {
        let todos = Todos::from_items(vec![
            labeled("login-page", None, &["ui"], &["auth"]),
            labeled("tokens", None, &["auth"], &[]),
            labeled("sessions", Some("me@x.io"), &["auth"], &[]),
            labeled("auth-docs", None, &["auth"], &["auth"]),
        ]);
        let page = todos.get("login-page").unwrap();
        assert_eq!(
            todos.waits_for(page),
            vec!["auth-docs", "sessions", "tokens"]
        );
        assert_eq!(todos.waits_summary(page), vec!["label auth"]);
        assert_eq!(todos.state(page), "blocked");
        assert_eq!(todos.needed_by("tokens"), vec!["auth-docs", "login-page"]);
        let docs = todos.get("auth-docs").unwrap();
        assert_eq!(todos.waits_for(docs), vec!["sessions", "tokens"]);
        let all: Vec<String> = todos.items.iter().map(|t| t.name.clone()).collect();
        assert_eq!(todos.ordered(&all).last().unwrap(), "login-page");
        assert!(todos.problems().is_empty());
    }

    #[test]
    fn finds_cycles_through_labels() {
        let todos = Todos::from_items(vec![
            labeled("a", None, &[], &["auth"]),
            todo("b", None, &["a"]),
        ]);
        assert!(todos.check_no_cycle(&["a".to_string()]).is_ok());
        let mut b = todos.get("b").unwrap().clone();
        b.set_labels(vec!["auth".into()]);
        let todos = todos.with(b);
        assert_eq!(todos.cycle_through("b").unwrap(), vec!["b", "a", "b"]);
        assert!(
            todos
                .check_no_cycle(&["b".to_string()])
                .unwrap_err()
                .contains("b -> a -> b")
        );
        assert!(todos.problems()[0].contains("dependency cycle"));
    }

    #[test]
    fn reports_a_label_dependency_without_todos() {
        let todos = Todos::from_items(vec![labeled(
            "a",
            None,
            &["auth"],
            &["auth", "gone", "Bad"],
        )]);
        let problems = todos.problems();
        assert_eq!(problems.len(), 3);
        assert!(problems[0].contains("dependsOnLabels names auth, which no other todo has"));
        assert_eq!(todos.state(todos.get("a").unwrap()), "available");
    }

    #[test]
    fn finds_cycles() {
        let todos = Todos::from_items(vec![
            todo("a", None, &["b"]),
            todo("b", None, &["c"]),
            todo("c", None, &["a"]),
        ]);
        assert!(todos.reaches("a", "c"));
        assert_eq!(todos.cycles().len(), 1);
        assert!(todos.problems()[0].contains("a -> b -> c -> a"));
    }

    #[test]
    fn orders_by_dependency() {
        let todos = Todos::from_items(vec![
            todo("a", None, &["c"]),
            todo("b", None, &[]),
            todo("c", None, &["b"]),
        ]);
        let all = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(todos.ordered(&all), vec!["b", "c", "a"]);
    }

    #[test]
    fn reports_problems() {
        let mut bad = todo("Bad_Name", Some("bob"), &["missing"]);
        bad.set_labels(vec!["Rust".into()]);
        let problems = Todos::from_items(vec![bad]).problems();
        assert_eq!(problems.len(), 4);
        assert!(problems[0].contains("use bad-name"));
    }

    #[test]
    fn checks_names() {
        assert!(check_name("fix-login-2").is_ok());
        assert_eq!(
            check_name("Fix_Login").unwrap_err(),
            "\"Fix_Login\" is not a todo name - use fix-login"
        );
        assert!(check_name("!!!").unwrap_err().contains("lowercase"));
        assert!(check_name("a--b").is_ok());
        assert!(check_name("a").is_ok());
        assert_eq!(
            check_name("-fix-login-").unwrap_err(),
            "\"-fix-login-\" is not a todo name - use fix-login"
        );
        assert!(check_name("---").unwrap_err().contains("lowercase"));
        assert_eq!(name_of(".todoer/todos/fix-login.md"), "fix-login");
        assert_eq!(name_of("fix-login"), "fix-login");
    }

    #[test]
    fn makes_slugs() {
        assert_eq!(slug("Fix: the  Login crash!", 60), "fix-the-login-crash");
        assert_eq!(slug("a very long title with many words", 16), "a-very-long");
        assert_eq!(slug("!!!", 10), "");
    }

    #[test]
    fn loads_the_todos_of_the_folder() {
        let root = std::env::temp_dir().join(format!("todoer-load-{}", std::process::id()));
        let dir = todos_dir(&root);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        let files = [
            (
                "a.md",
                "---\nassignee: me@x.io\nlabels: [x]\n---\n\nText.\n",
            ),
            ("b.md", "No front matter.\n"),
            ("notes.txt", "not a todo\n"),
            ("sub/c.md", "---\n---\n"),
        ];
        for (name, text) in files {
            std::fs::write(dir.join(name), text).unwrap();
        }
        let todos = Todos::load(&root).unwrap();
        let names: Vec<&str> = todos.items.iter().map(|todo| todo.name.as_str()).collect();
        assert_eq!(names, vec!["a", "b"]);
        assert_eq!(todos.get("a").unwrap().labels, vec!["x"]);
        assert_eq!(todos.state(todos.get("b").unwrap()), "available");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
