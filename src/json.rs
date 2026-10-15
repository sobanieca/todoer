//! A small JSON parser and writer for the settings files and the `--json` output.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    #[cfg(test)]
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

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

pub fn parse(text: &str) -> Result<Value, String> {
    let mut parser = Parser {
        chars: text.chars().collect(),
        at: 0,
    };
    parser.space();
    let value = parser.value()?;
    parser.space();
    if parser.at < parser.chars.len() {
        return Err(parser.error("unexpected text after the value"));
    }
    Ok(value)
}

struct Parser {
    chars: Vec<char>,
    at: usize,
}

impl Parser {
    fn error(&self, message: &str) -> String {
        let line = self.chars[..self.at.min(self.chars.len())]
            .iter()
            .filter(|c| **c == '\n')
            .count()
            + 1;
        format!("{message} on line {line}")
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn space(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.at += 1;
        }
    }

    fn expect(&mut self, word: &str) -> Result<(), String> {
        for c in word.chars() {
            if self.peek() != Some(c) {
                return Err(self.error(&format!("expected \"{word}\"")));
            }
            self.at += 1;
        }
        Ok(())
    }

    fn value(&mut self) -> Result<Value, String> {
        match self.peek() {
            Some('{') => self.object(),
            Some('[') => self.array(),
            Some('"') => self.string().map(Value::String),
            Some('t') => self.expect("true").map(|_| Value::Bool(true)),
            Some('f') => self.expect("false").map(|_| Value::Bool(false)),
            Some('n') => self.expect("null").map(|_| Value::Null),
            Some(c) if c == '-' || c.is_ascii_digit() => self.number(),
            _ => Err(self.error("expected a value")),
        }
    }

    fn object(&mut self) -> Result<Value, String> {
        self.at += 1;
        let mut entries = Vec::new();
        self.space();
        if self.peek() == Some('}') {
            self.at += 1;
            return Ok(Value::Object(entries));
        }
        loop {
            self.space();
            if self.peek() != Some('"') {
                return Err(self.error("expected a key"));
            }
            let key = self.string()?;
            self.space();
            self.expect(":")?;
            self.space();
            let value = self.value()?;
            entries.push((key, value));
            self.space();
            match self.peek() {
                Some(',') => self.at += 1,
                Some('}') => {
                    self.at += 1;
                    return Ok(Value::Object(entries));
                }
                _ => return Err(self.error("expected \",\" or \"}\"")),
            }
        }
    }

    fn array(&mut self) -> Result<Value, String> {
        self.at += 1;
        let mut items = Vec::new();
        self.space();
        if self.peek() == Some(']') {
            self.at += 1;
            return Ok(Value::Array(items));
        }
        loop {
            self.space();
            items.push(self.value()?);
            self.space();
            match self.peek() {
                Some(',') => self.at += 1,
                Some(']') => {
                    self.at += 1;
                    return Ok(Value::Array(items));
                }
                _ => return Err(self.error("expected \",\" or \"]\"")),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.at += 1;
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                return Err(self.error("the text ends inside a string"));
            };
            self.at += 1;
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let Some(escape) = self.peek() else {
                        return Err(self.error("the text ends inside a string"));
                    };
                    self.at += 1;
                    match escape {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'r' => out.push('\r'),
                        'b' => out.push('\u{8}'),
                        'f' => out.push('\u{c}'),
                        'u' => {
                            let code = self.hex()?;
                            if (0xd800..0xdc00).contains(&code) {
                                self.expect("\\u")?;
                                let low = self.hex()?;
                                let joined = 0x10000 + ((code - 0xd800) << 10) + (low - 0xdc00);
                                out.push(char::from_u32(joined).unwrap_or('\u{fffd}'));
                            } else {
                                out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                            }
                        }
                        other => out.push(other),
                    }
                }
                other => out.push(other),
            }
        }
    }

    fn hex(&mut self) -> Result<u32, String> {
        let digits: String = self.chars.iter().skip(self.at).take(4).collect();
        self.at += 4;
        u32::from_str_radix(&digits, 16).map_err(|_| self.error("bad \\u escape"))
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.at;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_digit() || "+-.eE".contains(c))
        {
            self.at += 1;
        }
        let text: String = self.chars[start..self.at].iter().collect();
        text.parse::<f64>()
            .map(Value::Number)
            .map_err(|_| self.error("bad number"))
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
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
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
    fn parses_objects() {
        let value = parse(r#"{"a": [1, true, null], "b": "x\"y\u00e9"}"#).unwrap();
        assert_eq!(
            value.get("a"),
            Some(&Value::Array(vec![
                Value::Number(1.0),
                Value::Bool(true),
                Value::Null
            ]))
        );
        assert_eq!(value.get("b"), Some(&Value::str("x\"y\u{e9}")));
    }

    #[test]
    fn reports_errors() {
        assert!(parse("{\"a\": }").unwrap_err().contains("line 1"));
        assert!(parse("[1,\n2").is_err());
    }

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
