//! A small JSON writer for the `--json` output.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    pub fn str(text: &str) -> Value {
        Value::String(text.to_string())
    }

    pub fn strings(items: &[String]) -> Value {
        Value::Array(items.iter().map(|item| Value::str(item)).collect())
    }

    pub fn opt(text: Option<&str>) -> Value {
        text.map_or(Value::Null, Value::str)
    }

    pub fn object(entries: Vec<(&str, Value)>) -> Value {
        Value::Object(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }
}

pub fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn stringify(value: &Value) -> String {
    let mut out = String::new();
    write(value, 0, &mut out);
    out
}

fn write(value: &Value, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth + 1);
    let close = "  ".repeat(depth);
    match value {
        Value::Null => out.push_str("null"),
        Value::Number(n) => {
            if n.fract() == 0.0 && n.abs() < 1e15 {
                out.push_str(&format!("{}", *n as i64));
            } else {
                out.push_str(&format!("{n}"));
            }
        }
        Value::String(s) => out.push_str(&quote(s)),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (index, item) in items.iter().enumerate() {
                out.push_str(&indent);
                write(item, depth + 1, out);
                if index + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&close);
            out.push(']');
        }
        Value::Object(entries) => {
            if entries.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (index, (key, item)) in entries.iter().enumerate() {
                out.push_str(&indent);
                out.push_str(&quote(key));
                out.push_str(": ");
                write(item, depth + 1, out);
                if index + 1 < entries.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&close);
            out.push('}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_values() {
        let value = Value::object(vec![
            ("a", Value::strings(&["x".to_string()])),
            ("n", Value::Number(3.0)),
        ]);
        assert_eq!(
            stringify(&value),
            "{\n  \"a\": [\n    \"x\"\n  ],\n  \"n\": 3\n}"
        );
    }
}
