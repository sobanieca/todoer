//! Label rules: the markdown files `.todoer/labels/<label>.md`. The file name is the label and the
//! body is the rule in plain words. todoer stores the rules. The agent reads them and adds labels.

use crate::project::{DATA_DIR, read_file, write_file};
use crate::todos::valid_label;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub fn rules_dir(root: &Path) -> PathBuf {
    root.join(DATA_DIR).join("labels")
}

pub fn rule_file(root: &Path, label: &str) -> PathBuf {
    rules_dir(root).join(format!("{label}.md"))
}

pub fn rule_path(label: &str) -> String {
    format!("{DATA_DIR}/labels/{label}.md")
}

fn entries(root: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(rules_dir(root)) else {
        return Vec::new();
    };
    let mut found: Vec<(String, PathBuf)> = entries
        .filter_map(Result::ok)
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            )
        })
        .filter(|(name, _)| !name.starts_with('.'))
        .collect();
    found.sort();
    found
}

pub struct Rules {
    pub items: BTreeMap<String, String>,
    problems: Vec<String>,
}

impl Rules {
    pub fn load(root: &Path) -> Result<Rules, String> {
        let mut items = BTreeMap::new();
        let mut problems = Vec::new();
        for (file_name, path) in entries(root) {
            let shown = format!("{DATA_DIR}/labels/{file_name}");
            let label = file_name.strip_suffix(".md").filter(|_| path.is_file());
            let Some(label) = label else {
                problems.push(format!("{shown}: not a rule - name the file <label>.md"));
                continue;
            };
            if !valid_label(label) {
                problems.push(format!(
                    "{shown}: \"{label}\" is not a label - use lowercase letters, digits and hyphens"
                ));
                continue;
            }
            let text = read_file(&path)?.trim().to_string();
            if text.is_empty() {
                problems.push(format!(
                    "{shown}: the rule is empty - write the rule, or run todoer labels rule {label} --rm"
                ));
            }
            items.insert(label.to_string(), text);
        }
        Ok(Rules { items, problems })
    }

    pub fn text(&self, label: &str) -> Option<&str> {
        self.items
            .get(label)
            .map(String::as_str)
            .filter(|text| !text.is_empty())
    }

    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    pub fn print_warnings(&self) {
        for problem in &self.problems {
            crate::logger::warn(problem);
        }
    }
}

pub fn write_rule(root: &Path, label: &str, text: &str) -> Result<(), String> {
    write_file(&rule_file(root, label), &format!("{}\n", text.trim()))
}

pub fn remove_rule(root: &Path, label: &str) -> Result<(), String> {
    std::fs::remove_file(rule_file(root, label))
        .map_err(|error| format!("cannot remove {}: {error}", rule_path(label)))
}

pub fn rename_rule(root: &Path, old: &str, new: &str) -> Result<(), String> {
    std::fs::rename(rule_file(root, old), rule_file(root, new))
        .map_err(|error| format!("cannot rename {}: {error}", rule_path(old)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rules_and_reports_bad_files() {
        let root = std::env::temp_dir().join(format!("todoer-rules-{}", std::process::id()));
        let dir = rules_dir(&root);
        std::fs::create_dir_all(dir.join("folder")).unwrap();
        std::fs::write(dir.join("auth.md"), "\nAdd it to the login work.\n\n").unwrap();
        std::fs::write(dir.join("empty.md"), "  \n").unwrap();
        std::fs::write(dir.join("Bad.md"), "A rule.\n").unwrap();
        std::fs::write(dir.join("notes.txt"), "Not a rule.\n").unwrap();
        std::fs::write(dir.join(".auth.md.1.tmp"), "").unwrap();
        let rules = Rules::load(&root).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(rules.text("auth"), Some("Add it to the login work."));
        assert!(rules.items.contains_key("empty"));
        assert_eq!(rules.text("empty"), None);
        assert!(!rules.items.contains_key("Bad"));
        assert_eq!(rules.items.len(), 2);
        assert_eq!(
            rules.problems(),
            [
                ".todoer/labels/Bad.md: \"Bad\" is not a label - use lowercase letters, digits and hyphens",
                ".todoer/labels/empty.md: the rule is empty - write the rule, or run todoer labels rule empty --rm",
                ".todoer/labels/folder: not a rule - name the file <label>.md",
                ".todoer/labels/notes.txt: not a rule - name the file <label>.md",
            ]
        );
    }
}
