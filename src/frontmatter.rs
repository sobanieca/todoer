//! Front matter: `key: value` lines between two `---` lines at the top of a markdown file. A change
//! keeps the order of the fields, the other lines and the body as they are.

use std::borrow::Cow;
use std::io::{self, BufRead};

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
    body_read: bool,
}

impl Document {
    pub fn new() -> Document {
        Document {
            lines: Vec::new(),
            body: String::new(),
            has_front: true,
            body_read: true,
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
                body_read: true,
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
            body_read: true,
        }
    }

    pub fn read_front(reader: &mut impl BufRead) -> io::Result<Option<Document>> {
        let start = reader.fill_buf()?;
        if !start.starts_with(b"---\n") && !start.starts_with(b"---\r\n") {
            return Ok(None);
        }
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let mut lines = Vec::new();
        loop {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                return Ok(None);
            }
            let text = line
                .strip_suffix('\n')
                .map(|text| text.strip_suffix('\r').unwrap_or(text))
                .unwrap_or(&line);
            if text == FENCE {
                break;
            }
            lines.push(front_line(text));
        }
        Ok(Some(Document {
            lines,
            body: String::new(),
            has_front: true,
            body_read: false,
        }))
    }

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

    pub fn set_text(&mut self, key: &str, value: &str) {
        self.set(key, &quote_if_needed(value));
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
        assert!(self.body_read, "render needs a document with its body");
        if !self.has_front {
            return self.body.clone();
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

fn quote_if_needed(value: &str) -> String {
    let value = value.replace('\n', " ");
    if needs_quotes(&value) {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value
    }
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
    fn quotes_text_when_needed() {
        let mut doc = Document::new();
        doc.set_text("title", "Fix: crash");
        doc.set_text("other", "plain words");
        assert_eq!(doc.get("title"), Some("\"Fix: crash\""));
        assert_eq!(doc.text("title").as_deref(), Some("Fix: crash"));
        assert_eq!(doc.get("other"), Some("plain words"));
    }

    fn front(text: &str) -> Option<Document> {
        Document::read_front(&mut text.as_bytes()).unwrap()
    }

    #[test]
    fn reads_only_the_front_matter() {
        let doc = front(TODO).unwrap();
        assert_eq!(doc.text("title").as_deref(), Some("Port: the parser"));
        assert_eq!(doc.list("labels"), vec!["a", "b c"]);
        assert_eq!(doc.body, "");
        let crlf = front("---\r\nstatus: todo\r\n---\r\nbody\r\n").unwrap();
        assert_eq!(crlf.get("status"), Some("todo"));
        assert!(front("---\n---\n").unwrap().lines.is_empty());
        assert!(front("---\nstatus: todo\n---").is_some());
        assert!(front("# Title\n").is_none());
        assert!(front("---").is_none());
        assert!(front("----\nstatus: todo\n---\n").is_none());
        assert!(front("---\nstatus: todo\nno end\n").is_none());
    }

    #[test]
    #[should_panic(expected = "render needs a document with its body")]
    fn a_front_matter_alone_does_not_render() {
        front(TODO).unwrap().render();
    }

    #[test]
    fn handles_an_empty_front_matter() {
        let doc = Document::parse("---\n---\nbody\n");
        assert!(doc.has_front());
        assert_eq!(doc.body, "body\n");
    }
}
