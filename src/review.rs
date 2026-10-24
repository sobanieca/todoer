//! The review file `review.md`: the comments on the latest patch.

use crate::diff;

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Pr,
    Folder(String),
    File(String),
    Line {
        path: String,
        start: usize,
        end: usize,
        old: bool,
    },
}

impl Target {
    pub fn path(&self) -> Option<&str> {
        match self {
            Target::Pr => None,
            Target::Folder(path) | Target::File(path) | Target::Line { path, .. } => Some(path),
        }
    }

    pub fn format(&self) -> String {
        match self {
            Target::Pr => "pr".to_string(),
            Target::Folder(path) => format!("{}/", path.trim_end_matches('/')),
            Target::File(path) => path.clone(),
            Target::Line {
                path,
                start,
                end,
                old,
            } => {
                let side = if *old { "old:" } else { "" };
                if start == end {
                    format!("{path}:{side}{start}")
                } else {
                    format!("{path}:{side}{start}-{end}")
                }
            }
        }
    }
}

fn parse_range(text: &str) -> Option<(usize, usize)> {
    let (start, end) = match text.split_once('-') {
        Some((start, end)) => (start.parse().ok()?, end.parse().ok()?),
        None => {
            let line = text.parse().ok()?;
            (line, line)
        }
    };
    (start >= 1 && end >= start).then_some((start, end))
}

pub fn parse_target(text: &str) -> Result<Target, String> {
    let value = text.trim().trim_start_matches("./");
    let bad = || {
        format!(
            "\"{text}\" is not a target - use path:line, path:start-end, path:old:line, a file or a folder/"
        )
    };
    if value.is_empty() || value.starts_with('/') || value.split('/').any(|part| part == "..") {
        return Err(bad());
    }
    if value.ends_with('/') {
        return Ok(Target::Folder(value.trim_end_matches('/').to_string()));
    }
    if let Some((path, rest)) = value.rsplit_once(":old:") {
        let (start, end) = parse_range(rest).ok_or_else(bad)?;
        return Ok(Target::Line {
            path: path.to_string(),
            start,
            end,
            old: true,
        });
    }
    if let Some((path, rest)) = value.rsplit_once(':')
        && rest.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        let (start, end) = parse_range(rest).ok_or_else(bad)?;
        return Ok(Target::Line {
            path: path.to_string(),
            start,
            end,
            old: false,
        });
    }
    Ok(Target::File(value.to_string()))
}

pub trait Tree {
    fn lines(&self, path: &str) -> Option<Vec<String>>;
    fn has_folder(&self, path: &str) -> bool;
}

fn missing(path: &str) -> String {
    format!("{path} is not in the pull request or in its base")
}

pub fn check_target(
    target: &Target,
    files: &[diff::File],
    tree: &dyn Tree,
) -> Result<Vec<String>, String> {
    match target {
        Target::Pr => Ok(Vec::new()),
        Target::Folder(path) => {
            let prefix = format!("{path}/");
            let changed = files
                .iter()
                .any(|file| file.paths().iter().any(|entry| entry.starts_with(&prefix)));
            if changed || tree.has_folder(path) {
                Ok(Vec::new())
            } else {
                Err(missing(&prefix))
            }
        }
        Target::File(path) => {
            if files.iter().any(|file| file.matches(path)) || tree.lines(path).is_some() {
                Ok(Vec::new())
            } else {
                Err(missing(path))
            }
        }
        Target::Line {
            path,
            start,
            end,
            old: true,
        } => {
            let file = files
                .iter()
                .find(|file| file.matches(path))
                .ok_or_else(|| format!("the pull request does not change {path}"))?;
            let mut quote = Vec::new();
            for number in *start..=*end {
                let line = file.find_line(true, number).ok_or_else(|| {
                        format!(
                            "line {number} of the old version of {path} is not in the diff - run todoer pr show to see the lines"
                        )
                    })?;
                quote.push(line.text.clone());
            }
            Ok(quote)
        }
        Target::Line {
            path, start, end, ..
        } => {
            let file = files.iter().find(|file| file.matches(path));
            if let Some(file) = file {
                let in_diff: Option<Vec<String>> = (*start..=*end)
                    .map(|number| file.find_line(false, number).map(|line| line.text.clone()))
                    .collect();
                if let Some(quote) = in_diff {
                    return Ok(quote);
                }
                if file.status == diff::Status::Deleted {
                    return Err(format!(
                        "the pull request deletes {path} - use {path}:old:N for a removed line"
                    ));
                }
            }
            let lines = tree.lines(path).ok_or_else(|| missing(path))?;
            if *end > lines.len() {
                return Err(format!(
                    "{path} has {} lines in the pull request - line {end} is not in it",
                    lines.len()
                ));
            }
            Ok(lines[start - 1..*end].to_vec())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub id: String,
    pub target: Target,
    pub author: String,
    pub quote: Vec<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Review {
    pub comments: Vec<Comment>,
}

fn parse_header(line: &str) -> Option<(String, Target, String)> {
    let rest = line.strip_prefix("### ")?;
    let (id, rest) = rest.split_once(' ')?;
    if !is_comment_id(id) {
        return None;
    }
    let (target, author) = if let Some(rest) = rest.strip_prefix('`') {
        let (target, author) = rest.split_once('`')?;
        (parse_target(target).ok()?, author)
    } else {
        (Target::Pr, rest.strip_prefix("pr")?)
    };
    Some((id.to_string(), target, author.trim().to_string()))
}

pub fn is_comment_id(id: &str) -> bool {
    id.len() > 1 && id.starts_with('c') && id[1..].chars().all(|c| c.is_ascii_digit())
}

impl Review {
    pub fn empty() -> Review {
        Review {
            comments: Vec::new(),
        }
    }

    pub fn parse(text: &str) -> Review {
        let mut review = Review::empty();
        let mut lines: Vec<String> = Vec::new();
        let mut current: Option<Comment> = None;
        let finish =
            |current: &mut Option<Comment>, lines: &mut Vec<String>, out: &mut Vec<Comment>| {
                if let Some(mut comment) = current.take() {
                    comment.text = lines.join("\n").trim().to_string();
                    out.push(comment);
                }
                lines.clear();
            };
        for line in text.replace("\r\n", "\n").lines() {
            if let Some((id, target, author)) = parse_header(line) {
                finish(&mut current, &mut lines, &mut review.comments);
                current = Some(Comment {
                    id,
                    target,
                    author,
                    quote: Vec::new(),
                    text: String::new(),
                });
                continue;
            }
            match current.as_mut() {
                None => {}
                Some(comment)
                    if lines.iter().all(|entry| entry.trim().is_empty())
                        && matches!(comment.target, Target::Line { .. })
                        && (line.starts_with("> ") || line == ">") =>
                {
                    comment
                        .quote
                        .push(line.strip_prefix("> ").unwrap_or("").to_string());
                }
                Some(_) => lines.push(
                    line.strip_prefix('\\')
                        .filter(|rest| rest.starts_with("### "))
                        .unwrap_or(line)
                        .to_string(),
                ),
            }
        }
        finish(&mut current, &mut lines, &mut review.comments);
        review
    }

    pub fn render(&self) -> String {
        let mut parts = Vec::new();
        for comment in &self.comments {
            let place = match &comment.target {
                Target::Pr => "pr".to_string(),
                target => format!("`{}`", target.format()),
            };
            let mut block = vec![format!("### {} {place} {}", comment.id, comment.author)];
            if !comment.quote.is_empty() {
                block.push(String::new());
                for line in &comment.quote {
                    block.push(if line.is_empty() {
                        ">".to_string()
                    } else {
                        format!("> {line}")
                    });
                }
            }
            block.push(String::new());
            for line in comment.text.trim().lines() {
                block.push(if line.starts_with("### ") {
                    format!("\\{line}")
                } else {
                    line.to_string()
                });
            }
            parts.push(block.join("\n"));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("{}\n", parts.join("\n\n"))
        }
    }

    pub fn next_id(&self) -> String {
        let max = self
            .comments
            .iter()
            .filter_map(|comment| comment.id[1..].parse::<usize>().ok())
            .max()
            .unwrap_or(0);
        format!("c{}", max + 1)
    }
}

pub fn clean_text(text: &str) -> Result<String, String> {
    let clean = text.trim();
    if clean.is_empty() {
        return Err("a comment needs text".into());
    }
    if clean.len() > 20_000 {
        return Err("a comment has 20000 characters or fewer".into());
    }
    Ok(clean.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_targets() {
        assert_eq!(parse_target("src/a.rs:42").unwrap().format(), "src/a.rs:42");
        assert_eq!(
            parse_target("src/a.rs:40-45").unwrap(),
            Target::Line {
                path: "src/a.rs".into(),
                start: 40,
                end: 45,
                old: false
            }
        );
        assert_eq!(
            parse_target("src/a.rs:old:12").unwrap().format(),
            "src/a.rs:old:12"
        );
        assert_eq!(
            parse_target("./src/cli/").unwrap(),
            Target::Folder("src/cli".into())
        );
        assert_eq!(
            parse_target("src/a.rs").unwrap(),
            Target::File("src/a.rs".into())
        );
        assert_eq!(
            parse_target("C:file").unwrap(),
            Target::File("C:file".into())
        );
        assert!(parse_target("src/a.rs:9-3").is_err());
        assert!(parse_target("../x").is_err());
        assert!(parse_target("/abs").is_err());
    }

    struct Base;

    impl Tree for Base {
        fn lines(&self, path: &str) -> Option<Vec<String>> {
            match path {
                "src/a.rs" => Some(vec!["one".into(), "TWO".into(), "three".into()]),
                "src/other.rs" => Some(vec!["x".into(), "y".into()]),
                _ => None,
            }
        }

        fn has_folder(&self, path: &str) -> bool {
            path == "lib"
        }
    }

    #[test]
    fn checks_targets_against_the_pull_request() {
        let files = diff::parse(
            "diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1,2 +1,2 @@\n one\n-two\n+TWO\n",
        );
        let check = |text: &str| check_target(&parse_target(text).unwrap(), &files, &Base);
        assert_eq!(check("src/a.rs:1-2").unwrap(), vec!["one", "TWO"]);
        assert_eq!(check("src/a.rs:old:2").unwrap(), vec!["two"]);
        assert_eq!(check("src/a.rs:3").unwrap(), vec!["three"]);
        assert_eq!(check("src/other.rs:2").unwrap(), vec!["y"]);
        assert!(check("src/a.rs:4").is_err());
        assert!(check("src/a.rs:old:3").is_err());
        assert!(check("src/").is_ok());
        assert!(check("lib/").is_ok());
        assert!(check("docs/").is_err());
        assert!(check("src/other.rs").is_ok());
        assert!(check("b.rs").is_err());
        assert!(check("b.rs:1").is_err());
    }

    #[test]
    fn round_trips_a_review() {
        let review = Review {
            comments: vec![
                Comment {
                    id: "c1".into(),
                    target: parse_target("src/a.rs:4-5").unwrap(),
                    author: "me@x.io".into(),
                    quote: vec!["let a = 1;".into(), "".into()],
                    text: "Use a constant.\n### not a header".into(),
                },
                Comment {
                    id: "c2".into(),
                    target: Target::Pr,
                    author: "me@x.io".into(),
                    quote: Vec::new(),
                    text: "Add tests.".into(),
                },
            ],
        };
        let text = review.render();
        assert!(text.contains("### c1 `src/a.rs:4-5` me@x.io\n\n> let a = 1;\n>\n\nUse a constant.\n\\### not a header"));
        assert!(text.contains("### c2 pr me@x.io\n"));
        assert_eq!(Review::parse(&text), review);
        assert_eq!(review.next_id(), "c3");
    }
}
