//! The todos: the markdown files `.todoer/todos/<name>.md` of the current checkout. The name is the
//! id and the title of a todo. The file is the text of the todo.

use crate::project::{DATA_DIR, read_file};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Todo {
    pub name: String,
    pub text: String,
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

pub fn remove_todo(root: &Path, name: &str) -> Result<(), String> {
    std::fs::remove_file(todo_file(root, name))
        .map_err(|error| format!("cannot remove {}: {error}", todo_path(name)))
}

pub struct Todos {
    pub items: Vec<Todo>,
    index: BTreeMap<String, usize>,
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
            items.push(Todo {
                name: name.to_string(),
                text: read_file(&path)?,
            });
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
        Todos { items, index }
    }

    pub fn get(&self, name: &str) -> Option<&Todo> {
        self.index.get(name).map(|at| &self.items[*at])
    }

    pub fn require(&self, name: &str) -> Result<&Todo, String> {
        self.get(name)
            .ok_or_else(|| format!("there is no todo {name} - run todoer ls"))
    }

    pub fn problems(&self) -> Vec<String> {
        self.items
            .iter()
            .filter_map(|todo| {
                check_name(&todo.name)
                    .err()
                    .map(|error| format!("{}: {error}", todo.name))
            })
            .collect()
    }

    pub fn print_warnings(&self) {
        for problem in self.problems() {
            crate::logger::warn(&problem);
        }
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

    #[test]
    fn reports_problems() {
        let todo = Todo {
            name: "Bad_Name".to_string(),
            text: String::new(),
        };
        let problems = Todos::from_items(vec![todo]).problems();
        assert_eq!(problems.len(), 1);
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
            ("a.md", "Text.\n"),
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
        assert_eq!(todos.get("a").unwrap().text, "Text.\n");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
