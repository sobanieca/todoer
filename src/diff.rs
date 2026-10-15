//! The parser for the combined diff of a pull request (`git diff -M`).

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Add,
    Del,
    Context,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub kind: Kind,
    pub old: Option<usize>,
    pub new: Option<usize>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub header: String,
    pub old_start: usize,
    pub old_count: usize,
    pub new_count: usize,
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
}

#[derive(Debug, Clone)]
pub struct File {
    pub path: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub status: Status,
    pub binary: bool,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<Hunk>,
}

impl File {
    pub fn letter(&self) -> char {
        match self.status {
            Status::Added => 'A',
            Status::Deleted => 'D',
            Status::Modified => 'M',
            Status::Renamed => 'R',
            Status::Copied => 'C',
        }
    }

    pub fn display_path(&self) -> String {
        match (&self.old_path, &self.new_path) {
            (Some(old), Some(new)) if old != new => format!("{old} -> {new}"),
            _ => self.path.clone(),
        }
    }

    pub fn paths(&self) -> Vec<&str> {
        let mut paths: Vec<&str> = Vec::new();
        for path in [&self.old_path, &self.new_path].into_iter().flatten() {
            if !paths.contains(&path.as_str()) {
                paths.push(path);
            }
        }
        if paths.is_empty() {
            paths.push(&self.path);
        }
        paths
    }

    pub fn matches(&self, path: &str) -> bool {
        self.paths().contains(&path)
    }

    pub fn find_line(&self, old_side: bool, number: usize) -> Option<&Line> {
        self.hunks.iter().flat_map(|hunk| &hunk.lines).find(|line| {
            if old_side {
                line.old == Some(number)
            } else {
                line.new == Some(number)
            }
        })
    }
}

fn unquote(path: &str) -> String {
    let Some(inner) = path
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    else {
        return path.to_string();
    };
    let mut bytes = Vec::new();
    let chars: Vec<char> = inner.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let c = chars[index];
        index += 1;
        if c != '\\' {
            let mut buffer = [0u8; 4];
            bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        let Some(&next) = chars.get(index) else {
            break;
        };
        if next.is_digit(8) {
            let digits: String = chars[index..(index + 3).min(chars.len())].iter().collect();
            bytes.push(u8::from_str_radix(&digits, 8).unwrap_or(b'?'));
            index += 3;
            continue;
        }
        index += 1;
        bytes.push(match next {
            'n' => b'\n',
            't' => b'\t',
            'r' => b'\r',
            other => other as u8,
        });
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn strip_prefix(path: &str) -> Option<String> {
    let clean = unquote(path.trim_end_matches('\t'));
    if clean == "/dev/null" {
        return None;
    }
    Some(
        clean
            .strip_prefix("a/")
            .or_else(|| clean.strip_prefix("b/"))
            .unwrap_or(&clean)
            .to_string(),
    )
}

fn header_paths(line: &str) -> (Option<String>, Option<String>) {
    let rest = &line["diff --git ".len()..];
    if rest.starts_with('"') {
        let mut escaped = false;
        for (at, c) in rest.char_indices().skip(1) {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                let (first, second) = rest.split_at(at + 1);
                return (strip_prefix(first), strip_prefix(second.trim_start()));
            }
        }
    }
    if rest.len() >= 5 && (rest.len() - 5).is_multiple_of(2) {
        let size = (rest.len() - 5) / 2;
        if rest.is_char_boundary(2 + size)
            && rest.is_char_boundary(size + 5)
            && rest[2..2 + size] == rest[size + 5..]
        {
            return (
                Some(rest[2..2 + size].to_string()),
                Some(rest[size + 5..].to_string()),
            );
        }
    }
    match rest.find(" b/") {
        Some(split) => (
            strip_prefix(&rest[..split]),
            Some(rest[split + 3..].to_string()),
        ),
        None => (None, None),
    }
}

fn parse_hunk_header(line: &str) -> Option<(usize, usize, usize, usize)> {
    let rest = line.strip_prefix("@@ -")?;
    let end = rest.find(" @@")?;
    let (old, new) = rest[..end].split_once(" +")?;
    let range = |text: &str| -> Option<(usize, usize)> {
        match text.split_once(',') {
            Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
            None => Some((text.parse().ok()?, 1)),
        }
    };
    let (old_start, old_count) = range(old)?;
    let (new_start, new_count) = range(new)?;
    Some((old_start, old_count, new_start, new_count))
}

pub fn parse(text: &str) -> Vec<File> {
    let mut files: Vec<File> = Vec::new();
    let mut in_hunk = false;
    let mut old_line = 0;
    let mut new_line = 0;
    for line in text.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.starts_with("diff --git ") {
            let (old_path, new_path) = header_paths(line);
            files.push(File {
                path: String::new(),
                old_path,
                new_path,
                status: Status::Modified,
                binary: false,
                additions: 0,
                deletions: 0,
                hunks: Vec::new(),
            });
            in_hunk = false;
            continue;
        }
        let Some(file) = files.last_mut() else {
            continue;
        };
        if in_hunk && let Some(hunk) = file.hunks.last_mut() {
            let (kind, rest) = match line.chars().next() {
                Some('+') => (Some(Kind::Add), &line[1..]),
                Some('-') => (Some(Kind::Del), &line[1..]),
                Some(' ') => (Some(Kind::Context), &line[1..]),
                Some('\\') => (None, ""),
                _ => {
                    in_hunk = false;
                    (None, "")
                }
            };
            if let Some(kind) = kind {
                let (old, new) = match kind {
                    Kind::Add => {
                        file.additions += 1;
                        new_line += 1;
                        (None, Some(new_line - 1))
                    }
                    Kind::Del => {
                        file.deletions += 1;
                        old_line += 1;
                        (Some(old_line - 1), None)
                    }
                    Kind::Context => {
                        old_line += 1;
                        new_line += 1;
                        (Some(old_line - 1), Some(new_line - 1))
                    }
                };
                hunk.lines.push(Line {
                    kind,
                    old,
                    new,
                    text: rest.to_string(),
                });
                continue;
            }
            if in_hunk {
                continue;
            }
        }
        if let Some((old_start, old_count, new_start, new_count)) = parse_hunk_header(line) {
            old_line = old_start;
            new_line = new_start;
            file.hunks.push(Hunk {
                header: line.to_string(),
                old_start,
                old_count,
                new_count,
                lines: Vec::new(),
            });
            in_hunk = true;
            continue;
        }
        if line.starts_with("new file mode") {
            file.status = Status::Added;
        } else if line.starts_with("deleted file mode") {
            file.status = Status::Deleted;
        } else if let Some(path) = line.strip_prefix("rename from ") {
            file.old_path = Some(unquote(path));
            file.status = Status::Renamed;
        } else if let Some(path) = line.strip_prefix("rename to ") {
            file.new_path = Some(unquote(path));
            file.status = Status::Renamed;
        } else if let Some(path) = line.strip_prefix("copy from ") {
            file.old_path = Some(unquote(path));
            file.status = Status::Copied;
        } else if let Some(path) = line.strip_prefix("copy to ") {
            file.new_path = Some(unquote(path));
            file.status = Status::Copied;
        } else if line.starts_with("Binary files ") || line == "GIT binary patch" {
            file.binary = true;
        } else if let Some(path) = line.strip_prefix("--- ") {
            file.old_path = strip_prefix(path);
        } else if let Some(path) = line.strip_prefix("+++ ") {
            file.new_path = strip_prefix(path);
        }
    }
    for file in &mut files {
        match file.status {
            Status::Added => file.old_path = None,
            Status::Deleted => file.new_path = None,
            _ => {}
        }
        file.path = file
            .new_path
            .clone()
            .or_else(|| file.old_path.clone())
            .unwrap_or_default();
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIFF: &str = "diff --git a/src/a.rs b/src/a.rs
index 1..2 100644
--- a/src/a.rs
+++ b/src/a.rs
@@ -1,3 +1,3 @@ fn main
 one
-two
+TWO
 three
@@ -10,0 +11,2 @@
+x
+y
diff --git a/new.txt b/new.txt
new file mode 100644
--- /dev/null
+++ b/new.txt
@@ -0,0 +1 @@
+hello
\\ No newline at end of file
diff --git a/old name.txt b/new name.txt
similarity index 90%
rename from old name.txt
rename to new name.txt
diff --git a/gone.txt b/gone.txt
deleted file mode 100644
--- a/gone.txt
+++ /dev/null
@@ -1 +0,0 @@
-bye
diff --git a/img.png b/img.png
Binary files a/img.png and b/img.png differ
";

    #[test]
    fn parses_files_and_lines() {
        let files = parse(DIFF);
        assert_eq!(files.len(), 5);
        let a = &files[0];
        assert_eq!(a.path, "src/a.rs");
        assert_eq!((a.additions, a.deletions), (3, 1));
        assert_eq!(a.hunks.len(), 2);
        assert_eq!(a.find_line(false, 2).unwrap().text, "TWO");
        assert_eq!(a.find_line(true, 2).unwrap().text, "two");
        assert_eq!(a.find_line(false, 12).unwrap().text, "y");
        assert_eq!(a.hunks[1].old_count, 0);
        assert_eq!(files[1].status, Status::Added);
        assert_eq!(files[1].old_path, None);
        assert_eq!(files[1].hunks[0].lines.len(), 1);
        assert_eq!(files[2].status, Status::Renamed);
        assert_eq!(files[2].display_path(), "old name.txt -> new name.txt");
        assert_eq!(files[3].status, Status::Deleted);
        assert_eq!(files[3].path, "gone.txt");
        assert!(files[4].binary);
    }

    #[test]
    fn unquotes_paths() {
        assert_eq!(unquote("\"a\\303\\251.txt\""), "a\u{e9}.txt");
        let files = parse("diff --git \"a/x y\\t.txt\" \"b/x y\\t.txt\"\n");
        assert_eq!(files[0].path, "x y\t.txt");
    }
}
