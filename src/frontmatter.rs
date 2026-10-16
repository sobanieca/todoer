//! Front matter: `key: value` lines between two `---` lines at the top of a markdown file. A change
//! keeps the order of the fields, the other lines and the body as they are. A front matter with no
//! line renders as nothing.

use std::borrow::Cow;

const FENCE: &str = "---";

#[derive(Debug, Clone, PartialEq)]
enum Line {
    Field(String, String),
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    lines: Vec<Line>,
    pub body: String,
    has_front: bool,
}

impl Document {
    pub fn new() -> Document {
        Document {
            lines: Vec::new(),
            body: String::new(),
            has_front: true,
        }
    }

    pub fn parse(text: &str) -> Document {
        let text: Cow<str> = if text.contains('\r') {
            Cow::Owned(text.replace("\r\n", "\n"))
        } else {
            Cow::Borrowed(text)
        };
        let Some((front, body)) = split_front(&text) else {
            return Document {
                lines: Vec::new(),
                body: text.into_owned(),
                has_front: false,
            };
        };
        let lines = if front.is_empty() {
            Vec::new()
        } else {
            front.split('\n').map(front_line).collect()
        };
        Document {
            lines,
            body: body.to_string(),
            has_front: true,
        }
    }

    #[cfg(test)]
    pub fn has_front(&self) -> bool {
        self.has_front
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.lines.iter().find_map(|line| match line {
            Line::Field(name, value) if name == key => Some(value.as_str()),
            _ => None,
        })
    }

    pub fn text(&self, key: &str) -> Option<String> {
        self.get(key).map(unquote).filter(|value| !value.is_empty())
    }

    pub fn list(&self, key: &str) -> Vec<String> {
        self.get(key).map(parse_list).unwrap_or_default()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.has_front = true;
        for line in &mut self.lines {
            if let Line::Field(name, current) = line
                && name == key
            {
                *current = value.to_string();
                return;
            }
        }
        self.lines
            .push(Line::Field(key.to_string(), value.to_string()));
    }

    pub fn set_list(&mut self, key: &str, items: &[String]) {
        if items.is_empty() {
            self.remove(key);
        } else {
            self.set(key, &format_list(items));
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.lines
            .retain(|line| !matches!(line, Line::Field(name, _) if name == key));
    }

    pub fn render(&self) -> String {
        if !self.has_front {
            return self.body.clone();
        }
        if self.lines.is_empty() {
            return self.body.trim_start_matches('\n').to_string();
        }
        let mut out = String::from("---\n");
        for line in &self.lines {
            match line {
                Line::Field(key, value) if value.is_empty() => out.push_str(&format!("{key}:\n")),
                Line::Field(key, value) => out.push_str(&format!("{key}: {value}\n")),
                Line::Other(raw) => out.push_str(&format!("{raw}\n")),
            }
        }
        out.push_str("---\n");
        out.push_str(&self.body);
        out
    }
}

fn split_front(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    if let Some(body) = rest.strip_prefix("---\n") {
        Some(("", body))
    } else if rest == FENCE {
        Some(("", ""))
    } else if let Some(end) = rest.find("\n---\n") {
        Some((&rest[..end], &rest[end + 5..]))
    } else {
        rest.strip_suffix("\n---").map(|front| (front, ""))
    }
}

fn front_line(line: &str) -> Line {
    match line.split_once(':') {
        Some((key, value))
            if !key.trim().is_empty()
                && !key.starts_with([' ', '\t', '-', '#'])
                && !key.contains(' ') =>
        {
            Line::Field(key.trim().to_string(), value.trim().to_string())
        }
        _ => Line::Other(line.to_string()),
    }
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    for quote in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
            let inner = &value[1..value.len() - 1];
            return if quote == '"' {
                inner.replace("\\\"", "\"").replace("\\\\", "\\")
            } else {
                inner.replace("''", "'")
            };
        }
    }
    value.to_string()
}

pub fn parse_list(value: &str) -> Vec<String> {
    let value = value.trim();
    let inner = value
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
        .unwrap_or(value);
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for c in inner.chars() {
        match (quote, c) {
            (None, '"' | '\'') => {
                quote = Some(c);
                current.push(c);
            }
            (Some(open), _) if c == open => {
                quote = None;
                current.push(c);
            }
            (None, ',') => {
                items.push(unquote(&current));
                current.clear();
            }
            _ => current.push(c),
        }
    }
    items.push(unquote(&current));
    items.into_iter().filter(|item| !item.is_empty()).collect()
}

fn needs_quotes(value: &str) -> bool {
    value.is_empty()
        || value.starts_with([
            '[', '{', '"', '\'', '#', '&', '*', '!', '|', '>', '%', '@', '`', ' ', '-',
        ])
        || value.ends_with(' ')
        || value.contains(": ")
        || value.contains(" #")
}

pub fn format_list(items: &[String]) -> String {
    let parts: Vec<String> = items
        .iter()
        .map(|item| {
            if needs_quotes(item) || item.contains([',', '[', ']']) {
                format!("\"{}\"", item.replace('\\', "\\\\").replace('"', "\\\""))
            } else {
                item.clone()
            }
        })
        .collect();
    format!("[{}]", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TODO: &str = "---\ntitle: Port: the parser\nstatus: todo\nlabels: [a, \"b c\"]\n# a note\n---\n\nBody text.\n";

    #[test]
    fn reads_fields_and_lists() {
        let doc = Document::parse(TODO);
        assert!(doc.has_front());
        assert_eq!(doc.text("title").as_deref(), Some("Port: the parser"));
        assert_eq!(doc.list("labels"), vec!["a", "b c"]);
        assert_eq!(doc.body, "\nBody text.\n");
    }

    #[test]
    fn keeps_the_file_when_nothing_changes() {
        assert_eq!(Document::parse(TODO).render(), TODO);
        let plain = "# Title\n\nno front matter\n";
        assert_eq!(Document::parse(plain).render(), plain);
        assert!(!Document::parse(plain).has_front());
    }

    #[test]
    fn changes_fields_in_place() {
        let mut doc = Document::parse(TODO);
        doc.set("status", "in-progress");
        doc.set("assignee", "me@example.com");
        doc.set_list("labels", &[]);
        assert_eq!(
            doc.render(),
            "---\ntitle: Port: the parser\nstatus: in-progress\n# a note\nassignee: me@example.com\n---\n\nBody text.\n"
        );
    }

    #[test]
    fn parses_list_forms() {
        assert_eq!(
            parse_list("[todo/a.md, todo/b.md]"),
            vec!["todo/a.md", "todo/b.md"]
        );
        assert_eq!(parse_list("a, b"), vec!["a", "b"]);
        assert_eq!(parse_list("['x,y', \"z\"]"), vec!["x,y", "z"]);
        assert!(parse_list("[]").is_empty());
        assert_eq!(format_list(&["a".into(), "x,y".into()]), "[a, \"x,y\"]");
    }

    #[test]
    fn handles_an_empty_front_matter() {
        let doc = Document::parse("---\n---\nbody\n");
        assert!(doc.has_front());
        assert_eq!(doc.body, "body\n");
        assert_eq!(doc.render(), "body\n");
        let mut doc = Document::parse("---\nassignee: me@x.io\n---\n\nText.\n");
        doc.remove("assignee");
        assert_eq!(doc.render(), "Text.\n");
    }
}
