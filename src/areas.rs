//! Code areas: named parts of the code in `.todoer/areas`, and a matcher for the gitignore
//! pattern rules.

use crate::diff;
use crate::project::DATA_DIR;
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
    area: String,
    pattern: String,
    dir_only: bool,
}

impl Rule {
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

fn parse(text: &str) -> Vec<Rule> {
    let mut rules = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((area, patterns)) = line.split_once(':') else {
            continue;
        };
        let area = area.trim();
        if area.is_empty() {
            continue;
        }
        for raw in patterns
            .split_whitespace()
            .take_while(|part| !part.starts_with('#'))
        {
            let dir_only = raw.ends_with('/') && raw.len() > 1;
            let trimmed = raw.trim_end_matches('/');
            let anchored = trimmed.starts_with('/') || trimmed.contains('/');
            let body = trimmed.trim_start_matches('/');
            let pattern = if anchored {
                body.to_string()
            } else {
                format!("**/{body}")
            };
            rules.push(Rule {
                area: area.to_string(),
                pattern,
                dir_only,
            });
        }
    }
    rules
}

#[derive(Default)]
pub struct Areas {
    rules: Vec<Rule>,
}

pub struct Count {
    pub area: Option<String>,
    pub files: usize,
    pub additions: usize,
    pub deletions: usize,
}

impl Areas {
    pub fn load(root: &Path) -> Areas {
        let text = std::fs::read_to_string(root.join(DATA_DIR).join("areas")).unwrap_or_default();
        Areas {
            rules: parse(&text),
        }
    }

    pub fn of_path(&self, path: &str) -> Option<&str> {
        self.rules
            .iter()
            .rev()
            .find(|rule| rule.matches(path))
            .map(|rule| rule.area.as_str())
    }

    pub fn of_file(&self, file: &diff::File) -> Vec<Option<&str>> {
        let mut areas = Vec::new();
        for path in file.paths() {
            let area = self.of_path(path);
            if !areas.contains(&area) {
                areas.push(area);
            }
        }
        areas
    }

    fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        for rule in &self.rules {
            if !names.contains(&rule.area.as_str()) {
                names.push(&rule.area);
            }
        }
        names
    }

    pub fn count(&self, files: &[&diff::File]) -> Vec<Count> {
        if self.rules.is_empty() {
            return Vec::new();
        }
        let mut order: Vec<Option<&str>> = self.names().into_iter().map(Some).collect();
        order.push(None);
        let mut counts: Vec<Count> = order
            .iter()
            .map(|area| Count {
                area: area.map(str::to_string),
                files: 0,
                additions: 0,
                deletions: 0,
            })
            .collect();
        for file in files {
            for area in self.of_file(file) {
                let at = order.iter().position(|item| *item == area).unwrap_or(0);
                counts[at].files += 1;
                counts[at].additions += file.additions;
                counts[at].deletions += file.deletions;
            }
        }
        counts.retain(|count| count.files > 0);
        counts
    }
}

pub struct Filter<'a> {
    pub areas: &'a Areas,
    only: Vec<String>,
    hide: Vec<String>,
}

fn names(values: &[String], option: &str) -> Result<Vec<String>, String> {
    let mut names: Vec<String> = Vec::new();
    for name in values.iter().flat_map(|value| value.split(',')) {
        let name = name.trim();
        if !name.is_empty() && !names.iter().any(|known| known == name) {
            names.push(name.to_string());
        }
    }
    if !values.is_empty() && names.is_empty() {
        return Err(format!(
            "--{option} needs a name - use: --{option} security,tests"
        ));
    }
    Ok(names)
}

fn list(word: &str, names: &[String]) -> String {
    let plural = if names.len() == 1 { "" } else { "s" };
    format!("{word}{plural} {}", names.join(", "))
}

impl<'a> Filter<'a> {
    pub fn new(areas: &'a Areas, only: &[String], hide: &[String]) -> Result<Filter<'a>, String> {
        let only = names(only, "area")?;
        let hide = names(hide, "exclude")?;
        let known = areas.names();
        if (!only.is_empty() || !hide.is_empty()) && known.is_empty() {
            return Err(format!(
                "there are no areas - name them in {DATA_DIR}/areas, one per line: \"<area>: <pattern>\""
            ));
        }
        if let Some(unknown) = only
            .iter()
            .chain(&hide)
            .find(|name| !known.contains(&name.as_str()))
        {
            return Err(format!(
                "there is no area {unknown} - the areas are: {}",
                known.join(", ")
            ));
        }
        Ok(Filter { areas, only, hide })
    }

    pub fn is_all(&self) -> bool {
        self.only.is_empty() && self.hide.is_empty()
    }

    pub fn keeps_file(&self, file: &diff::File) -> bool {
        self.is_all()
            || self
                .areas
                .of_file(file)
                .into_iter()
                .any(|area| self.selects(area))
    }

    pub fn keeps_path(&self, path: &str) -> bool {
        self.is_all() || self.selects(self.areas.of_path(path))
    }

    fn selects(&self, area: Option<&str>) -> bool {
        let named = |names: &[String]| area.is_some_and(|area| names.iter().any(|n| n == area));
        (self.only.is_empty() || named(&self.only)) && !named(&self.hide)
    }

    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if !self.only.is_empty() {
            parts.push(list("area", &self.only));
        }
        if !self.hide.is_empty() {
            let word = if parts.is_empty() { "all but" } else { "not" };
            parts.push(format!("{word} {}", list("area", &self.hide)));
        }
        parts.join(", ")
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

    fn areas(text: &str) -> Areas {
        Areas { rules: parse(text) }
    }

    #[test]
    fn the_last_matching_area_wins() {
        let areas = areas(
            "# areas\nall: *\nrust: *.rs\ndocs: /docs/ *.md # text\nlegacy: src/legacy/\nno pattern\ntests: /tests\ne2e-tests: /tests/e2e\n",
        );
        assert_eq!(areas.of_path("README.txt"), Some("all"));
        assert_eq!(areas.of_path("README.md"), Some("docs"));
        assert_eq!(areas.of_path("src/main.rs"), Some("rust"));
        assert_eq!(areas.of_path("docs/a/b.rs"), Some("docs"));
        assert_eq!(areas.of_path("src/legacy/x.rs"), Some("legacy"));
        assert_eq!(areas.of_path("docs"), Some("all"));
        assert_eq!(areas.of_path("tests/unit/a.rs"), Some("tests"));
        assert_eq!(areas.of_path("tests/e2e/login.rs"), Some("e2e-tests"));
        assert_eq!(areas.of_path("text"), Some("all"));
    }

    #[test]
    fn a_path_can_have_no_area() {
        let areas = areas("api: /src/api\n");
        assert_eq!(areas.of_path("src/api/auth/login.rs"), Some("api"));
        assert_eq!(areas.of_path("src/main.rs"), None);
        assert!(Areas::default().count(&[]).is_empty());
    }

    #[test]
    fn filters_by_area() {
        let areas = areas(
            "tests: /tests\ne2e-tests: /tests/e2e\nsecurity: /src/api/auth\nunit-tests: *.test.js\n",
        );
        assert_eq!(
            areas.of_path("src/api/auth/login.test.js"),
            Some("unit-tests")
        );
        assert_eq!(areas.of_path("src/api/auth/login.js"), Some("security"));
        let only =
            |values: &[&str]| -> Vec<String> { values.iter().map(|v| v.to_string()).collect() };
        let filter = Filter::new(&areas, &only(&["security,e2e-tests"]), &[]).unwrap();
        assert!(filter.keeps_path("src/api/auth/login.js"));
        assert!(filter.keeps_path("tests/e2e/login.rs"));
        assert!(!filter.keeps_path("tests/unit.rs"));
        assert!(!filter.keeps_path("README.md"));
        assert_eq!(filter.describe(), "areas security, e2e-tests");
        let filter = Filter::new(&areas, &[], &only(&["e2e-tests", "unit-tests"])).unwrap();
        assert!(filter.keeps_path("README.md"));
        assert!(filter.keeps_path("tests/unit.rs"));
        assert!(!filter.keeps_path("tests/e2e/login.rs"));
        assert!(!filter.keeps_path("src/app.test.js"));
        assert_eq!(filter.describe(), "all but areas e2e-tests, unit-tests");
        let filter = Filter::new(&areas, &only(&["tests"]), &only(&["unit-tests"])).unwrap();
        assert!(filter.keeps_path("tests/a.rs"));
        assert!(!filter.keeps_path("tests/a.test.js"));
        assert_eq!(filter.describe(), "area tests, not area unit-tests");
        assert!(
            Filter::new(&areas, &[], &[])
                .unwrap()
                .keeps_path("README.md")
        );
        assert!(Filter::new(&areas, &[], &only(&["nope"])).is_err());
        assert!(Filter::new(&Areas::default(), &only(&["tests"]), &[]).is_err());
    }
}
