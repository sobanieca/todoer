//! Code owners: teams in `.todoer/teams/<team>.md`, rules in `.todoer/codeowners` or
//! `.github/CODEOWNERS`, and a matcher for the gitignore pattern rules.

use crate::args::Args;
use crate::diff;
use crate::project::{DATA_DIR, Project};
use std::collections::BTreeMap;
use std::path::Path;

pub fn glob(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    matches(&pattern, &text)
}

fn matches(pattern: &[char], text: &[char]) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }
    if pattern.starts_with(&['*', '*']) {
        let rest = &pattern[2..];
        if let Some(rest) = rest.strip_prefix(&['/']) {
            if matches(rest, text) {
                return true;
            }
            return (0..text.len()).any(|at| text[at] == '/' && matches(rest, &text[at + 1..]));
        }
        return (0..=text.len()).any(|at| matches(rest, &text[at..]));
    }
    match pattern[0] {
        '*' => {
            for at in 0..=text.len() {
                if matches(&pattern[1..], &text[at..]) {
                    return true;
                }
                if at < text.len() && text[at] == '/' {
                    break;
                }
            }
            false
        }
        '?' => !text.is_empty() && text[0] != '/' && matches(&pattern[1..], &text[1..]),
        '[' => match class(pattern) {
            Some((end, test)) => {
                !text.is_empty()
                    && text[0] != '/'
                    && test(text[0])
                    && matches(&pattern[end..], &text[1..])
            }
            None => !text.is_empty() && text[0] == '[' && matches(&pattern[1..], &text[1..]),
        },
        '\\' if pattern.len() > 1 => {
            !text.is_empty() && text[0] == pattern[1] && matches(&pattern[2..], &text[1..])
        }
        c => !text.is_empty() && text[0] == c && matches(&pattern[1..], &text[1..]),
    }
}

type ClassTest = Box<dyn Fn(char) -> bool>;

fn class(pattern: &[char]) -> Option<(usize, ClassTest)> {
    let mut at = 1;
    let negate = matches!(pattern.get(at), Some('!') | Some('^'));
    if negate {
        at += 1;
    }
    let mut ranges: Vec<(char, char)> = Vec::new();
    let mut first = true;
    while at < pattern.len() {
        let c = pattern[at];
        if c == ']' && !first {
            let test = move |value: char| {
                ranges
                    .iter()
                    .any(|(low, high)| *low <= value && value <= *high)
                    != negate
            };
            return Some((at + 1, Box::new(test)));
        }
        first = false;
        if pattern.get(at + 1) == Some(&'-') && pattern.get(at + 2).is_some_and(|end| *end != ']') {
            ranges.push((c, pattern[at + 2]));
            at += 3;
        } else {
            ranges.push((c, c));
            at += 1;
        }
    }
    None
}

struct Rule {
    pattern: String,
    dir_only: bool,
    owners: Vec<String>,
}

impl Rule {
    fn parse(line: &str, teams: &BTreeMap<String, Vec<String>>) -> Option<Rule> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let mut parts = line.split_whitespace();
        let raw = parts.next()?;
        let owners = parts
            .take_while(|part| !part.starts_with('#'))
            .map(|owner| owner_name(owner, teams))
            .collect();
        let dir_only = raw.ends_with('/') && raw.len() > 1;
        let trimmed = raw.trim_end_matches('/');
        let anchored = trimmed.starts_with('/') || trimmed.contains('/');
        let body = trimmed.trim_start_matches('/');
        let pattern = if anchored {
            body.to_string()
        } else {
            format!("**/{body}")
        };
        Some(Rule {
            pattern,
            dir_only,
            owners,
        })
    }

    fn matches(&self, path: &str) -> bool {
        let mut candidates: Vec<&str> =
            path.match_indices('/').map(|(at, _)| &path[..at]).collect();
        if !self.dir_only {
            candidates.push(path);
        }
        candidates
            .iter()
            .any(|candidate| glob(&self.pattern, candidate))
    }
}

fn owner_name(owner: &str, teams: &BTreeMap<String, Vec<String>>) -> String {
    match owner.strip_prefix('@') {
        Some(name) => {
            let team = name.rsplit('/').next().unwrap_or(name);
            if teams.contains_key(team) || name.contains('/') {
                format!("@{team}")
            } else {
                format!("@{name}")
            }
        }
        None => owner.to_string(),
    }
}

fn read_members(text: &str) -> Vec<String> {
    let mut members = Vec::new();
    for word in text.split_whitespace() {
        let clean = word.trim_matches(|c: char| {
            !(c.is_ascii_alphanumeric() || c == '@' || c == '.' || c == '_' || c == '+' || c == '-')
        });
        let clean = clean.trim_start_matches('-').trim_end_matches(['.', '-']);
        if clean.contains('@') && !clean.starts_with('@') && !members.contains(&clean.to_string()) {
            members.push(clean.to_string());
        }
    }
    members
}

#[derive(Default)]
pub struct Owners {
    rules: Vec<Rule>,
    teams: BTreeMap<String, Vec<String>>,
    pub configured: bool,
}

impl Owners {
    pub fn load(root: &Path) -> Owners {
        let mut teams = BTreeMap::new();
        if let Ok(entries) = std::fs::read_dir(root.join(DATA_DIR).join("teams")) {
            for entry in entries.filter_map(Result::ok) {
                let name = entry.file_name().to_string_lossy().into_owned();
                let Some(team) = name.strip_suffix(".md") else {
                    continue;
                };
                if let Ok(text) = std::fs::read_to_string(entry.path()) {
                    teams.insert(team.to_string(), read_members(&text));
                }
            }
        }
        let text = std::fs::read_to_string(root.join(DATA_DIR).join("codeowners"))
            .or_else(|_| std::fs::read_to_string(root.join(".github/CODEOWNERS")));
        let configured = text.is_ok();
        let rules = text
            .unwrap_or_default()
            .lines()
            .filter_map(|line| Rule::parse(line, &teams))
            .collect();
        Owners {
            rules,
            teams,
            configured,
        }
    }

    pub fn of_path(&self, path: &str) -> Vec<String> {
        self.rules
            .iter()
            .rev()
            .find(|rule| rule.matches(path))
            .map(|rule| rule.owners.clone())
            .unwrap_or_default()
    }

    pub fn of_file(&self, file: &diff::File) -> Vec<String> {
        let mut owners: Vec<String> = Vec::new();
        for path in file.paths() {
            for owner in self.of_path(path) {
                if !owners.contains(&owner) {
                    owners.push(owner);
                }
            }
        }
        owners
    }

    pub fn identities(&self, email: &str) -> Vec<String> {
        let mut out = vec![email.to_string()];
        for (team, members) in &self.teams {
            if members
                .iter()
                .any(|member| member.eq_ignore_ascii_case(email))
            {
                out.push(format!("@{team}"));
            }
        }
        out
    }
}

pub enum Filter {
    All,
    Owner(Vec<String>),
    Mine(Vec<String>),
    Unowned,
}

impl Filter {
    pub fn from_args(args: &Args, project: &Project, owners: &Owners) -> Result<Filter, String> {
        let chosen = ["owner", "mine", "unowned"]
            .iter()
            .filter(|name| args.has(name))
            .count();
        if chosen > 1 {
            return Err("use one of --owner, --mine and --unowned".into());
        }
        if let Some(owner) = args.value("owner") {
            let name = if owner.contains('@') && !owner.starts_with('@') {
                owner.to_string()
            } else {
                format!("@{}", owner.trim_start_matches('@'))
            };
            let mut names = vec![name.clone()];
            if !name.starts_with('@') {
                names = owners.identities(&name);
            }
            return Ok(Filter::Owner(names));
        }
        if args.has("mine") {
            return Ok(Filter::Mine(owners.identities(&project.me()?)));
        }
        if args.has("unowned") {
            return Ok(Filter::Unowned);
        }
        Ok(Filter::All)
    }

    pub fn is_all(&self) -> bool {
        matches!(self, Filter::All)
    }

    pub fn keep(&self, file_owners: &[String]) -> bool {
        match self {
            Filter::All => true,
            Filter::Owner(names) | Filter::Mine(names) => file_owners
                .iter()
                .any(|owner| names.iter().any(|name| name.eq_ignore_ascii_case(owner))),
            Filter::Unowned => file_owners.is_empty(),
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Filter::All => "all files".into(),
            Filter::Owner(names) => format!("owner {}", names[0]),
            Filter::Mine(_) => "my files".into(),
            Filter::Unowned => "files with no owner".into(),
        }
    }
}

pub fn display(owners: &[String], configured: bool) -> String {
    if !configured {
        String::new()
    } else if owners.is_empty() {
        "(no owner)".into()
    } else {
        owners.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_globs() {
        assert!(glob("*.rs", "main.rs"));
        assert!(!glob("*.rs", "src/main.rs"));
        assert!(glob("**/*.rs", "src/main.rs"));
        assert!(glob("**/*.rs", "main.rs"));
        assert!(glob("src/**", "src/a/b.rs"));
        assert!(glob("a/**/b", "a/b"));
        assert!(glob("a/**/b", "a/x/y/b"));
        assert!(!glob("a/**/b", "a/x/yb"));
        assert!(glob("file?.txt", "file1.txt"));
        assert!(!glob("file?.txt", "file/.txt"));
        assert!(glob("[abc].md", "b.md"));
        assert!(!glob("[!abc].md", "b.md"));
        assert!(glob("[a-c]x", "cx"));
        assert!(glob("\\*.md", "*.md"));
        assert!(!glob("\\*.md", "a.md"));
    }

    fn owners(text: &str, teams: &[(&str, &[&str])]) -> Owners {
        let teams: BTreeMap<String, Vec<String>> = teams
            .iter()
            .map(|(name, members)| {
                (
                    name.to_string(),
                    members.iter().map(|m| m.to_string()).collect(),
                )
            })
            .collect();
        Owners {
            rules: text
                .lines()
                .filter_map(|line| Rule::parse(line, &teams))
                .collect(),
            teams,
            configured: true,
        }
    }

    #[test]
    fn the_last_matching_rule_wins() {
        let owners = owners(
            "# owners\n* @everyone\n*.rs @backend\n/docs/ docs@x.io\nsrc/legacy/ @org/qa\nbuild\n",
            &[("backend", &["a@x.io"]), ("qa", &[])],
        );
        assert_eq!(owners.of_path("README.md"), vec!["@everyone"]);
        assert_eq!(owners.of_path("src/main.rs"), vec!["@backend"]);
        assert_eq!(owners.of_path("docs/a/b.rs"), vec!["docs@x.io"]);
        assert_eq!(owners.of_path("src/legacy/x.rs"), vec!["@qa"]);
        assert_eq!(owners.of_path("docs"), vec!["@everyone"]);
        assert!(owners.of_path("tools/build/out.txt").is_empty());
        assert_eq!(owners.identities("a@x.io"), vec!["a@x.io", "@backend"]);
    }

    #[test]
    fn reads_team_members() {
        assert_eq!(
            read_members(
                "# Backend\n\n- a@x.io\n- <b@x.io>\n* agent-1@bots.dev, lead\n@notamember\n"
            ),
            vec!["a@x.io", "b@x.io", "agent-1@bots.dev"]
        );
    }

    #[test]
    fn filters_owners() {
        let file_owners = vec!["@backend".to_string()];
        assert!(Filter::Mine(vec!["a@x.io".into(), "@backend".into()]).keep(&file_owners));
        assert!(!Filter::Owner(vec!["@qa".into()]).keep(&file_owners));
        assert!(Filter::Unowned.keep(&[]));
        assert!(!Filter::Unowned.keep(&file_owners));
    }
}
