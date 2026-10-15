//! The todos: markdown files with a `status` field in the front matter. The path relative to the
//! repository root is the id of a todo.

use crate::frontmatter::Document;
use crate::git;
use crate::project::{DATA_DIR, read_file, write_file};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub const STATUSES: [&str; 4] = ["todo", "in-progress", "done", "dropped"];

#[derive(Debug, Clone)]
pub struct Todo {
    pub path: String,
    pub doc: Document,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    pub depends_on: Vec<String>,
}

impl Todo {
    pub fn from_document(path: &str, doc: Document) -> Option<Todo> {
        Todo::build(path, doc, |doc| heading(doc.body.lines()))
    }

    fn build(
        path: &str,
        doc: Document,
        body_heading: impl FnOnce(&Document) -> Option<String>,
    ) -> Option<Todo> {
        if !doc.has_front() {
            return None;
        }
        let status = doc.text("status")?;
        let title = doc
            .text("title")
            .or_else(|| body_heading(&doc))
            .unwrap_or_else(|| stem(path));
        Some(Todo {
            path: path.to_string(),
            title,
            status,
            assignee: doc.text("assignee"),
            labels: doc.list("labels"),
            depends_on: doc.list("dependsOn"),
            doc,
        })
    }

    pub fn is_closed(&self) -> bool {
        self.status == "done" || self.status == "dropped"
    }

    pub fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
        self.doc.set("status", status);
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

    pub fn save(&self, root: &Path) -> Result<(), String> {
        write_file(&root.join(&self.path), &self.doc.render())
    }
}

fn heading<S: AsRef<str>>(mut lines: impl Iterator<Item = S>) -> Option<String> {
    lines
        .find_map(|line| {
            line.as_ref()
                .strip_prefix("# ")
                .map(|title| title.trim().to_string())
        })
        .filter(|title| !title.is_empty())
}

fn load_todo(root: &Path, path: &str) -> Option<Todo> {
    let mut reader = BufReader::new(File::open(root.join(path)).ok()?);
    let doc = Document::read_front(&mut reader).ok()??;
    Todo::build(path, doc, |_| heading(reader.lines().map_while(Result::ok)))
}

fn stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.strip_suffix(".md").unwrap_or(name).to_string()
}

pub fn valid_label(label: &str) -> bool {
    !label.is_empty()
        && label
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn check_label(label: &str) -> Result<(), String> {
    if valid_label(label) {
        Ok(())
    } else {
        Err(format!(
            "\"{label}\" is not a label - use lowercase letters, digits and hyphens"
        ))
    }
}

pub fn read_todo(root: &Path, path: &str) -> Result<Todo, String> {
    let file = root.join(path);
    if !file.is_file() {
        return Err(format!("there is no todo {path} - run todoer ls"));
    }
    let doc = Document::parse(&read_file(&file)?);
    Todo::from_document(path, doc)
        .ok_or_else(|| format!("{path} is not a todo - it has no status field in its front matter"))
}

pub struct Todos {
    pub items: Vec<Todo>,
    index: BTreeMap<String, usize>,
}

impl Todos {
    pub fn load(root: &Path) -> Result<Todos, String> {
        let output = git::check(
            root,
            &["ls-files", "-co", "--exclude-standard", "-z", "--", "*.md"],
        )?;
        let paths: BTreeSet<&str> = output
            .stdout
            .split('\0')
            .filter(|path| !path.is_empty())
            .filter(|path| !path.starts_with(&format!("{DATA_DIR}/")))
            .collect();
        let items = paths
            .into_iter()
            .filter_map(|path| load_todo(root, path))
            .collect();
        Ok(Todos::from_items(items))
    }

    pub fn from_items(mut items: Vec<Todo>) -> Todos {
        items.sort_by(|a, b| a.path.cmp(&b.path));
        let index = items
            .iter()
            .enumerate()
            .map(|(at, todo)| (todo.path.clone(), at))
            .collect();
        Todos { items, index }
    }

    pub fn get(&self, path: &str) -> Option<&Todo> {
        self.index.get(path).map(|at| &self.items[*at])
    }

    pub fn require(&self, path: &str) -> Result<&Todo, String> {
        self.get(path)
            .ok_or_else(|| format!("there is no todo {path} - run todoer ls"))
    }

    pub fn waits_for(&self, todo: &Todo) -> Vec<String> {
        todo.depends_on
            .iter()
            .filter(|path| **path != todo.path)
            .filter(|path| self.get(path).is_some_and(|other| !other.is_closed()))
            .cloned()
            .collect()
    }

    pub fn state(&self, todo: &Todo) -> &'static str {
        match todo.status.as_str() {
            "done" => "done",
            "dropped" => "dropped",
            "in-progress" => "in-progress",
            _ if self.waits_for(todo).is_empty() => "ready",
            _ => "blocked",
        }
    }

    pub fn needed_by(&self, path: &str) -> Vec<String> {
        self.items
            .iter()
            .filter(|todo| todo.depends_on.iter().any(|dep| dep == path))
            .map(|todo| todo.path.clone())
            .collect()
    }

    pub fn reaches(&self, from: &str, to: &str) -> bool {
        let mut seen = BTreeSet::new();
        let mut stack = vec![from.to_string()];
        while let Some(path) = stack.pop() {
            if path == to {
                return true;
            }
            if !seen.insert(path.clone()) {
                continue;
            }
            if let Some(todo) = self.get(&path) {
                stack.extend(todo.depends_on.iter().cloned());
            }
        }
        false
    }

    pub fn cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut done = BTreeSet::new();
        for todo in &self.items {
            let mut stack = Vec::new();
            self.visit(&todo.path, &mut stack, &mut done, &mut cycles);
        }
        cycles
    }

    fn visit(
        &self,
        path: &str,
        stack: &mut Vec<String>,
        done: &mut BTreeSet<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if let Some(at) = stack.iter().position(|entry| entry == path) {
            let mut cycle = stack[at..].to_vec();
            cycle.push(path.to_string());
            cycles.push(cycle);
            return;
        }
        if done.contains(path) {
            return;
        }
        let Some(todo) = self.get(path) else {
            return;
        };
        stack.push(path.to_string());
        for dep in &todo.depends_on {
            self.visit(dep, stack, done, cycles);
        }
        stack.pop();
        done.insert(path.to_string());
    }

    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        for todo in &self.items {
            if !STATUSES.contains(&todo.status.as_str()) {
                problems.push(format!(
                    "{}: unknown status \"{}\" - use todo, in-progress, done or dropped",
                    todo.path, todo.status
                ));
            }
            for label in &todo.labels {
                if !valid_label(label) {
                    problems.push(format!(
                        "{}: bad label \"{label}\" - use lowercase letters, digits and hyphens",
                        todo.path
                    ));
                }
            }
            if let Some(assignee) = &todo.assignee
                && !assignee.contains('@')
            {
                problems.push(format!(
                    "{}: bad assignee \"{assignee}\" - use a git email",
                    todo.path
                ));
            }
            for dep in &todo.depends_on {
                if dep == &todo.path {
                    problems.push(format!("{}: dependsOn names the todo itself", todo.path));
                } else if self.get(dep).is_none() {
                    problems.push(format!(
                        "{}: dependsOn names {dep}, which is not a todo",
                        todo.path
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

    pub fn ordered(&self, paths: &[String]) -> Vec<String> {
        let wanted: BTreeSet<&String> = paths.iter().collect();
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for path in paths {
            self.order_visit(path, &wanted, &mut seen, &mut out);
        }
        out
    }

    fn order_visit(
        &self,
        path: &str,
        wanted: &BTreeSet<&String>,
        seen: &mut BTreeSet<String>,
        out: &mut Vec<String>,
    ) {
        if !seen.insert(path.to_string()) {
            return;
        }
        if let Some(todo) = self.get(path) {
            for dep in &todo.depends_on {
                if wanted.contains(dep) {
                    self.order_visit(dep, wanted, seen, out);
                }
            }
        }
        out.push(path.to_string());
    }
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

    fn todo(path: &str, status: &str, deps: &[&str]) -> Todo {
        let mut doc = Document::new();
        doc.set("status", status);
        let deps: Vec<String> = deps.iter().map(|dep| dep.to_string()).collect();
        doc.set_list("dependsOn", &deps);
        Todo::from_document(path, doc).unwrap()
    }

    #[test]
    fn finds_states() {
        let todos = Todos::from_items(vec![
            todo("a.md", "todo", &["b.md"]),
            todo("b.md", "in-progress", &[]),
            todo("c.md", "todo", &["d.md"]),
            todo("d.md", "dropped", &[]),
        ]);
        assert_eq!(todos.state(todos.get("a.md").unwrap()), "blocked");
        assert_eq!(todos.state(todos.get("b.md").unwrap()), "in-progress");
        assert_eq!(todos.state(todos.get("c.md").unwrap()), "ready");
        assert_eq!(todos.needed_by("b.md"), vec!["a.md"]);
    }

    #[test]
    fn finds_cycles() {
        let todos = Todos::from_items(vec![
            todo("a.md", "todo", &["b.md"]),
            todo("b.md", "todo", &["c.md"]),
            todo("c.md", "todo", &["a.md"]),
        ]);
        assert!(todos.reaches("a.md", "c.md"));
        assert_eq!(todos.cycles().len(), 1);
        assert!(todos.problems()[0].contains("a.md -> b.md -> c.md -> a.md"));
    }

    #[test]
    fn orders_by_dependency() {
        let todos = Todos::from_items(vec![
            todo("a.md", "todo", &["c.md"]),
            todo("b.md", "todo", &[]),
            todo("c.md", "todo", &["b.md"]),
        ]);
        let all = vec!["a.md".to_string(), "b.md".to_string(), "c.md".to_string()];
        assert_eq!(todos.ordered(&all), vec!["b.md", "c.md", "a.md"]);
    }

    #[test]
    fn reports_problems() {
        let mut bad = todo("x.md", "later", &["missing.md"]);
        bad.set_labels(vec!["Rust".into()]);
        bad.set_assignee(Some("bob"));
        let problems = Todos::from_items(vec![bad]).problems();
        assert_eq!(problems.len(), 4);
    }

    #[test]
    fn makes_slugs() {
        assert_eq!(slug("Fix: the  Login crash!", 60), "fix-the-login-crash");
        assert_eq!(slug("a very long title with many words", 16), "a-very-long");
        assert_eq!(slug("!!!", 10), "");
    }

    #[test]
    fn takes_the_title_from_the_heading() {
        let doc = Document::parse("---\nstatus: todo\n---\n\n# Heading title\n");
        assert_eq!(
            Todo::from_document("x/y.md", doc).unwrap().title,
            "Heading title"
        );
        let doc = Document::parse("---\nstatus: todo\n---\n");
        assert_eq!(Todo::from_document("x/y.md", doc).unwrap().title, "y");
        assert!(Todo::from_document("z.md", Document::parse("---\ntitle: x\n---\n")).is_none());
    }

    #[test]
    fn loads_the_todos_from_the_front_matter() {
        let root = std::env::temp_dir().join(format!("todoer-load-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let files = [
            (
                "a.md",
                "---\nstatus: todo\nlabels: [x]\n---\n\nText.\n\n# Heading\n",
            ),
            ("b.md", "---\nstatus: done\ntitle: B\n---\n# Not used\n"),
            ("c.md", "---\ntitle: no status\n---\n"),
            ("d.md", "# Notes\n"),
        ];
        for (name, text) in files {
            std::fs::write(root.join(name), text).unwrap();
        }
        let a = load_todo(&root, "a.md").unwrap();
        assert_eq!(
            (a.title.as_str(), a.labels.clone()),
            ("Heading", vec!["x".into()])
        );
        assert_eq!(load_todo(&root, "b.md").unwrap().title, "B");
        assert!(load_todo(&root, "c.md").is_none());
        assert!(load_todo(&root, "d.md").is_none());
        assert!(load_todo(&root, "missing.md").is_none());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
