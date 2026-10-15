//! The text of `pr summary` and `pr details`.

use crate::colors;
use crate::diff::{self, Kind};
use crate::git;
use crate::owners::{self, Filter, Owners};
use crate::project::Project;
use crate::prs::Pr;
use crate::review::{Comment, Review, Target};

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

fn short(hash: &str) -> &str {
    &hash[..hash.len().min(7)]
}

pub fn file_comments<'a>(review: &'a Review, file: &diff::File) -> Vec<&'a Comment> {
    review
        .comments
        .iter()
        .filter(|comment| {
            comment.target.path().is_some_and(|path| {
                !matches!(comment.target, Target::Folder(_)) && file.matches(path)
            })
        })
        .collect()
}

pub fn summary(project: &Project, pr: &Pr, owners: &Owners, filter: &Filter) -> String {
    let mut out = String::new();
    let review = pr.review();
    let files = pr.files();
    out.push_str(&format!(
        "{}  {}   {}  by {}\n",
        colors::bold(&pr.slug),
        pr.title,
        colors::green("open"),
        pr.author
    ));
    let mut second = vec![format!("base {}", short(&pr.base))];
    if !pr.todos.is_empty() {
        second.push(format!("todos: {}", pr.todos.join(", ")));
    }
    if review.verdict != "pending" {
        second.push(verdict_color(&review.verdict));
    }
    out.push_str(&format!("{}\n", second.join("   ")));
    if !pr.description().is_empty() {
        out.push('\n');
        for line in pr.description().lines() {
            out.push_str(&format!("{}\n", format!("  {line}").trim_end()));
        }
    }
    out.push('\n');
    let shown: Vec<(&diff::File, Vec<String>)> = files
        .iter()
        .map(|file| (file, owners.of_file(file)))
        .filter(|(_, file_owners)| filter.keep(file_owners))
        .collect();
    let unchanged: Vec<(String, Vec<String>)> = unchanged_paths(&review, &files)
        .into_iter()
        .map(|path| {
            let path_owners = owners.of_path(&path);
            (path, path_owners)
        })
        .filter(|(_, path_owners)| filter.keep(path_owners))
        .collect();
    let path_width = shown
        .iter()
        .map(|(file, _)| file.display_path().chars().count())
        .chain(unchanged.iter().map(|(path, _)| path.chars().count()))
        .max()
        .unwrap_or(0);
    let adds_width = shown
        .iter()
        .map(|(file, _)| file.additions.to_string().len() + 1)
        .max()
        .unwrap_or(0);
    let dels_width = shown
        .iter()
        .map(|(file, _)| file.deletions.to_string().len() + 1)
        .max()
        .unwrap_or(0);
    let stats_width = (adds_width + dels_width + 1).max(if unchanged.is_empty() { 0 } else { 9 });
    for (file, file_owners) in &shown {
        let count = file_comments(&review, file).len();
        let adds = if file.additions > 0 {
            colors::green(&format!("+{}", file.additions))
        } else {
            String::new()
        };
        let dels = if file.deletions > 0 {
            colors::red(&format!("-{}", file.deletions))
        } else {
            String::new()
        };
        let comments = if count > 0 {
            plural(count, "comment")
        } else {
            String::new()
        };
        let stats = format!(
            "{} {}",
            colors::pad_left(&adds, adds_width),
            colors::pad_left(&dels, dels_width)
        );
        let line = format!(
            " {}  {}  {}  {}  {}",
            letter_color(file.letter()),
            colors::pad(&file.display_path(), path_width),
            colors::pad(&stats, stats_width),
            colors::pad(&colors::yellow(&comments), 10),
            colors::gray(&owners::display(file_owners, owners.configured))
        );
        out.push_str(&format!("{}\n", line.trim_end()));
    }
    for (path, path_owners) in &unchanged {
        let count = path_comments(&review, path).len();
        let line = format!(
            "    {}  {}  {}  {}",
            colors::pad(path, path_width),
            colors::pad(&colors::gray("no change"), stats_width),
            colors::pad(&colors::yellow(&plural(count, "comment")), 10),
            colors::gray(&owners::display(path_owners, owners.configured))
        );
        out.push_str(&format!("{}\n", line.trim_end()));
    }
    let additions: usize = shown.iter().map(|(file, _)| file.additions).sum();
    let deletions: usize = shown.iter().map(|(file, _)| file.deletions).sum();
    let mut footer = vec![
        format!("{}, +{additions} -{deletions}", plural(shown.len(), "file")),
        plural(pr.commits(), "commit"),
    ];
    if !review.comments.is_empty() {
        footer.push(plural(review.comments.len(), "comment"));
    }
    let drafts = pr.drafts(project).len();
    if drafts > 0 {
        footer.push(colors::yellow(&plural(drafts, "draft comment")));
    }
    out.push_str(&format!("{}\n", footer.join("    ")));
    if !filter.is_all() {
        out.push_str(&colors::gray(&format!(
            "showing {}: {} of {} files\n",
            filter.describe(),
            shown.len(),
            files.len()
        )));
    }
    out
}

pub fn verdict_color(verdict: &str) -> String {
    match verdict {
        "approved" => colors::green(verdict),
        "changes-requested" => colors::red(verdict),
        _ => colors::gray(verdict),
    }
}

fn letter_color(letter: char) -> String {
    let text = letter.to_string();
    match letter {
        'A' => colors::green(&text),
        'D' => colors::red(&text),
        'R' | 'C' => colors::magenta(&text),
        _ => colors::yellow(&text),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Row {
    Hunk(String),
    Gap,
    Line {
        old: Option<usize>,
        new: Option<usize>,
        kind: Kind,
        text: String,
    },
}

pub fn rows(file: &diff::File, old_lines: Option<&[String]>, context: usize) -> Vec<Row> {
    let Some(old_lines) = old_lines.filter(|_| context > 0) else {
        let mut rows = Vec::new();
        for hunk in &file.hunks {
            rows.push(Row::Hunk(hunk.header.clone()));
            rows.extend(hunk.lines.iter().map(|line| Row::Line {
                old: line.old,
                new: line.new,
                kind: line.kind,
                text: line.text.clone(),
            }));
        }
        return rows;
    };
    let mut rows = Vec::new();
    let mut shown_until = 0usize;
    let mut offset = 0isize;
    let context_line = |number: usize, offset: isize| Row::Line {
        old: Some(number),
        new: Some((number as isize + offset) as usize),
        kind: Kind::Context,
        text: old_lines.get(number - 1).cloned().unwrap_or_default(),
    };
    for (index, hunk) in file.hunks.iter().enumerate() {
        let first_old = if hunk.old_count == 0 {
            hunk.old_start + 1
        } else {
            hunk.old_start
        };
        let start = (shown_until + 1)
            .max(first_old.saturating_sub(context))
            .max(1);
        if start > shown_until + 1 {
            rows.push(Row::Gap);
        }
        for number in start..first_old {
            rows.push(context_line(number, offset));
        }
        rows.extend(hunk.lines.iter().map(|line| Row::Line {
            old: line.old,
            new: line.new,
            kind: line.kind,
            text: line.text.clone(),
        }));
        shown_until = if hunk.old_count == 0 {
            hunk.old_start
        } else {
            hunk.old_start + hunk.old_count - 1
        };
        offset += hunk.new_count as isize - hunk.old_count as isize;
        let next_first = file.hunks.get(index + 1).map(|next| {
            if next.old_count == 0 {
                next.old_start + 1
            } else {
                next.old_start
            }
        });
        let mut end = (shown_until + context).min(old_lines.len());
        if let Some(next_first) = next_first {
            end = end.min(next_first.saturating_sub(1));
        }
        for number in shown_until + 1..=end {
            rows.push(context_line(number, offset));
        }
        shown_until = shown_until.max(end);
    }
    if shown_until < old_lines.len() && !file.hunks.is_empty() {
        rows.push(Row::Gap);
    }
    rows
}

pub fn filter_lines(rows: Vec<Row>, start: usize, end: usize) -> Vec<Row> {
    let mut last_new = 0;
    let mut kept: Vec<Row> = Vec::new();
    for row in rows {
        match &row {
            Row::Line { new, .. } => {
                let position = new.unwrap_or(last_new + 1);
                if let Some(new) = new {
                    last_new = *new;
                }
                if (start..=end).contains(&position) {
                    kept.push(row);
                }
            }
            _ => kept.push(row),
        }
    }
    let mut out: Vec<Row> = Vec::new();
    for row in kept {
        if !matches!(row, Row::Line { .. })
            && out
                .last()
                .is_some_and(|last| !matches!(last, Row::Line { .. }))
        {
            out.pop();
        }
        out.push(row);
    }
    while out
        .last()
        .is_some_and(|last| !matches!(last, Row::Line { .. }))
    {
        out.pop();
    }
    out
}

pub fn old_file(project: &Project, pr: &Pr, file: &diff::File) -> Option<Vec<String>> {
    let path = file.old_path.as_ref()?;
    let text = git::show(&project.root, &pr.base, path).ok()??;
    Some(text.lines().map(str::to_string).collect())
}

pub fn base_lines(project: &Project, pr: &Pr, path: &str) -> Option<Vec<String>> {
    let text = git::show(&project.root, &pr.base, path).ok()??;
    Some(text.lines().map(str::to_string).collect())
}

fn whole_rows(project: &Project, pr: &Pr, file: &diff::File) -> Option<Vec<Row>> {
    if file.binary {
        return None;
    }
    let old = match file.old_path {
        None => Vec::new(),
        Some(_) => old_file(project, pr, file)?,
    };
    Some(
        rows(file, Some(&old), old.len() + 1)
            .into_iter()
            .filter(|row| matches!(row, Row::Line { .. }))
            .collect(),
    )
}

pub fn new_lines(project: &Project, pr: &Pr, file: &diff::File) -> Option<Vec<String>> {
    let rows = whole_rows(project, pr, file)?;
    Some(
        rows.into_iter()
            .filter_map(|row| match row {
                Row::Line {
                    new: Some(_), text, ..
                } => Some(text),
                _ => None,
            })
            .collect(),
    )
}

fn unchanged_rows(lines: &[String]) -> Vec<Row> {
    lines
        .iter()
        .enumerate()
        .map(|(index, text)| Row::Line {
            old: Some(index + 1),
            new: Some(index + 1),
            kind: Kind::Context,
            text: text.clone(),
        })
        .collect()
}

const SNIPPET_CONTEXT: usize = 2;

fn first_line(rows: &[Row]) -> usize {
    rows.iter()
        .find_map(|row| match row {
            Row::Line { new, old, .. } => new.or(*old),
            _ => None,
        })
        .unwrap_or(usize::MAX)
}

fn last_line(rows: &[Row]) -> usize {
    rows.iter()
        .filter_map(|row| match row {
            Row::Line { new, .. } => *new,
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

fn shown(rows: &[Row], comment: &Comment) -> bool {
    let Target::Line { end, old: side, .. } = &comment.target else {
        return true;
    };
    rows.iter().any(|row| match row {
        Row::Line { old, new, .. } => (if *side { old } else { new }) == &Some(*end),
        _ => false,
    })
}

fn outside_comments<'a>(
    comments: &[&'a Comment],
    rows: &[Row],
    lines: Option<(usize, usize)>,
) -> Vec<&'a Comment> {
    comments
        .iter()
        .copied()
        .filter(|comment| match &comment.target {
            Target::Line {
                end, old: false, ..
            } => lines.is_none_or(|(first, last)| (first..=last).contains(end)),
            _ => false,
        })
        .filter(|comment| !shown(rows, comment))
        .collect()
}

fn with_snippets(file_rows: Vec<Row>, all_rows: &[Row], comments: &[&Comment]) -> Vec<Row> {
    let mut ranges: Vec<(usize, usize)> = comments
        .iter()
        .filter_map(|comment| match &comment.target {
            Target::Line { start, end, .. } => Some((
                start.saturating_sub(SNIPPET_CONTEXT).max(1),
                end + SNIPPET_CONTEXT,
            )),
            _ => None,
        })
        .collect();
    ranges.sort();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        match merged.last_mut() {
            Some(last) if start <= last.1 + 1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    let in_diff: Vec<usize> = file_rows
        .iter()
        .filter_map(|row| match row {
            Row::Line { new, .. } => *new,
            _ => None,
        })
        .collect();
    let mut blocks: Vec<(usize, bool, Vec<Row>)> = Vec::new();
    for row in file_rows {
        let starts_block = !matches!(row, Row::Line { .. });
        match blocks.last_mut() {
            Some((_, false, rows)) if !starts_block => rows.push(row),
            _ => blocks.push((0, false, vec![row])),
        }
    }
    for block in &mut blocks {
        block.0 = first_line(&block.2);
    }
    for (start, end) in merged {
        let rows: Vec<Row> = all_rows
            .iter()
            .filter(|row| {
                matches!(row, Row::Line { new: Some(n), .. } if (start..=end).contains(n) && !in_diff.contains(n))
            })
            .cloned()
            .collect();
        if !rows.is_empty() {
            blocks.push((first_line(&rows), true, rows));
        }
    }
    blocks.sort_by_key(|block| block.0);
    let mut out: Vec<Row> = Vec::new();
    let mut ends_with_snippet = false;
    for (start, snippet, rows) in blocks {
        let next_to_last = last_line(&out) + 1 == start;
        let gap = match out.last() {
            None => start > 1,
            Some(Row::Gap) => false,
            Some(_) => !next_to_last,
        };
        if snippet && gap {
            out.push(Row::Gap);
        }
        if matches!(rows.first(), Some(Row::Gap)) && matches!(out.last(), Some(Row::Gap)) {
            out.extend(rows.into_iter().skip(1));
        } else {
            out.extend(rows);
        }
        ends_with_snippet = snippet;
    }
    if ends_with_snippet && last_line(&out) < last_line(all_rows) {
        out.push(Row::Gap);
    }
    out
}

fn unchanged_paths(review: &Review, files: &[diff::File]) -> Vec<String> {
    let mut paths: Vec<String> = review
        .comments
        .iter()
        .filter(|comment| matches!(comment.target, Target::File(_) | Target::Line { .. }))
        .filter_map(|comment| comment.target.path().map(str::to_string))
        .filter(|path| !files.iter().any(|file| file.matches(path)))
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

fn path_comments<'a>(review: &'a Review, path: &str) -> Vec<&'a Comment> {
    review
        .comments
        .iter()
        .filter(|comment| {
            !matches!(comment.target, Target::Folder(_)) && comment.target.path() == Some(path)
        })
        .collect()
}

pub fn parse_lines(text: &str) -> Result<(usize, usize), String> {
    let bad = || format!("--lines needs a range like 40-80, not \"{text}\"");
    let (start, end) = text.split_once('-').ok_or_else(bad)?;
    let start: usize = start.trim().parse().map_err(|_| bad())?;
    let end: usize = end.trim().parse().map_err(|_| bad())?;
    if end < start {
        return Err(bad());
    }
    Ok((start, end))
}

pub struct DetailsOptions<'a> {
    pub paths: Vec<String>,
    pub context: usize,
    pub lines: Option<(usize, usize)>,
    pub filter: &'a Filter,
}

fn comment_lines(comment: &Comment, drafts: &[String], indent: &str) -> Vec<String> {
    let mut out = Vec::new();
    let draft = if drafts.contains(&comment.id) {
        colors::gray(" (draft)")
    } else {
        String::new()
    };
    let head = format!("[{}]{draft} {}: ", comment.id, comment.author);
    let width = colors::width(&head);
    for (index, line) in comment.text.lines().enumerate() {
        let prefix = if index == 0 {
            head.clone()
        } else {
            " ".repeat(width)
        };
        if line.trim().is_empty() {
            out.push(indent.trim_end().to_string());
            continue;
        }
        out.push(format!(
            "{indent}{}",
            colors::yellow(&format!("{prefix}{line}"))
        ));
    }
    if comment.text.is_empty() {
        out.push(format!("{indent}{}", colors::yellow(head.trim_end())));
    }
    out
}

struct Printer<'a> {
    drafts: &'a [String],
    indent: String,
    out: String,
    printed: Vec<String>,
}

impl Printer<'_> {
    fn comment(&mut self, comment: &Comment) {
        for line in comment_lines(comment, self.drafts, &self.indent) {
            self.out.push_str(&format!("{line}\n"));
        }
        self.printed.push(comment.id.clone());
    }

    fn header(&mut self, path: &str, stats: &str, path_owners: &str) {
        let header = format!(
            "{}  ({stats})  {}",
            colors::bold(path),
            colors::gray(path_owners)
        );
        self.out.push_str(&format!("{}\n", header.trim_end()));
    }

    fn rows(&mut self, rows: &[Row], comments: &[&Comment]) {
        let gutter = " ".repeat(10);
        for row in rows {
            match row {
                Row::Hunk(header) => self
                    .out
                    .push_str(&format!("{gutter} | {}\n", colors::cyan(header))),
                Row::Gap => self
                    .out
                    .push_str(&format!("{gutter} | {}\n", colors::gray("..."))),
                Row::Line {
                    old,
                    new,
                    kind,
                    text,
                } => {
                    let number =
                        |value: &Option<usize>| value.map(|n| n.to_string()).unwrap_or_default();
                    let numbers = format!("{:>5}{:>5}", number(old), number(new));
                    let body = match kind {
                        Kind::Add => colors::green(&format!("+ {text}")),
                        Kind::Del => colors::red(&format!("- {text}")),
                        Kind::Context => format!("  {text}"),
                    };
                    let line = format!("{} | {}", colors::gray(&numbers), body);
                    self.out.push_str(&format!("{}\n", line.trim_end()));
                    for comment in comments {
                        let Target::Line {
                            end, old: old_side, ..
                        } = &comment.target
                        else {
                            continue;
                        };
                        let at = if *old_side { old } else { new };
                        if *at == Some(*end) && !self.printed.contains(&comment.id) {
                            self.comment(comment);
                        }
                    }
                }
            }
        }
    }
}

pub fn details(
    project: &Project,
    pr: &Pr,
    owners: &Owners,
    options: &DetailsOptions,
) -> Result<String, String> {
    let files = pr.files();
    let review = pr.review();
    let drafts: Vec<String> = pr
        .drafts(project)
        .into_iter()
        .map(|comment| comment.id)
        .collect();
    let unchanged = unchanged_paths(&review, &files);
    for path in &options.paths {
        if !files.iter().any(|file| file.matches(path)) && !unchanged.contains(path) {
            return Err(format!(
                "the pull request does not change {path}, and no comment names it"
            ));
        }
    }
    let gutter = " ".repeat(10);
    let mut printer = Printer {
        drafts: &drafts,
        indent: format!("{gutter} |   "),
        out: String::new(),
        printed: Vec::new(),
    };
    if options.paths.is_empty() {
        let general: Vec<&Comment> = review
            .comments
            .iter()
            .filter(|comment| matches!(comment.target, Target::Pr | Target::Folder(_)))
            .collect();
        for comment in general {
            let heading = match &comment.target {
                Target::Folder(_) => format!("comment on {}", comment.target.format()),
                _ => "comment on the pull request".to_string(),
            };
            printer
                .out
                .push_str(&format!("{}\n", colors::bold(&heading)));
            for line in comment_lines(comment, &drafts, "  ") {
                printer.out.push_str(&format!("{line}\n"));
            }
            printer.out.push('\n');
        }
    }
    let wanted = |path: &str| options.paths.is_empty() || options.paths.iter().any(|p| p == path);
    let mut first = true;
    for file in &files {
        if !options.paths.is_empty() && !options.paths.iter().any(|path| file.matches(path)) {
            continue;
        }
        let file_owners = owners.of_file(file);
        if !options.filter.keep(&file_owners) {
            continue;
        }
        if !first {
            printer.out.push('\n');
        }
        first = false;
        let mut stats = vec![file.letter().to_string()];
        if file.additions > 0 || file.deletions > 0 {
            stats.push(format!("+{} -{}", file.additions, file.deletions));
        }
        printer.header(
            &file.display_path(),
            &stats.join(", "),
            &owners::display(&file_owners, owners.configured),
        );
        let comments = file_comments(&review, file);
        for comment in comments
            .iter()
            .filter(|comment| matches!(comment.target, Target::File(_)))
        {
            printer.comment(comment);
        }
        if file.binary {
            printer
                .out
                .push_str(&format!("{gutter} |   {}\n", colors::gray("binary file")));
            continue;
        }
        let old_lines = if options.context > 0 {
            old_file(project, pr, file)
        } else {
            None
        };
        let empty: Vec<String> = Vec::new();
        let old_ref = match (&old_lines, file.old_path.is_none()) {
            (Some(lines), _) => Some(lines.as_slice()),
            (None, true) => Some(empty.as_slice()),
            _ => None,
        };
        let mut file_rows = rows(file, old_ref, options.context);
        if let Some((start, end)) = options.lines {
            file_rows = filter_lines(file_rows, start, end);
        }
        let outside = outside_comments(&comments, &file_rows, options.lines);
        if !outside.is_empty()
            && let Some(all_rows) = whole_rows(project, pr, file)
        {
            file_rows = with_snippets(file_rows, &all_rows, &outside);
        }
        printer.rows(&file_rows, &comments);
    }
    for path in unchanged.iter().filter(|path| wanted(path)) {
        let path_owners = owners.of_path(path);
        if !options.filter.keep(&path_owners) {
            continue;
        }
        if !first {
            printer.out.push('\n');
        }
        first = false;
        printer.header(
            path,
            "no change",
            &owners::display(&path_owners, owners.configured),
        );
        let comments = path_comments(&review, path);
        for comment in comments
            .iter()
            .filter(|comment| matches!(comment.target, Target::File(_)))
        {
            printer.comment(comment);
        }
        let lines = base_lines(project, pr, path).unwrap_or_default();
        let outside = outside_comments(&comments, &[], options.lines);
        let file_rows = with_snippets(Vec::new(), &unchanged_rows(&lines), &outside);
        printer.rows(&file_rows, &comments);
    }
    let mut out = printer.out;
    if first && !files.is_empty() {
        out.push_str(&format!("no file matches {}\n", options.filter.describe()));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file() -> diff::File {
        diff::parse(
            "diff --git a/a.txt b/a.txt\n--- a/a.txt\n+++ b/a.txt\n@@ -3,1 +3,2 @@\n-c\n+C\n+D\n@@ -8,0 +10,1 @@\n+X\n",
        )
        .remove(0)
    }

    fn numbers(rows: &[Row]) -> Vec<String> {
        rows.iter()
            .map(|row| match row {
                Row::Hunk(_) => "@".to_string(),
                Row::Gap => "...".to_string(),
                Row::Line { old, new, .. } => format!(
                    "{}/{}",
                    old.map(|n| n.to_string()).unwrap_or_default(),
                    new.map(|n| n.to_string()).unwrap_or_default()
                ),
            })
            .collect()
    }

    #[test]
    fn adds_context_from_the_old_file() {
        let old: Vec<String> = (1..=12).map(|n| format!("line {n}")).collect();
        let rows = rows(&file(), Some(&old), 1);
        assert_eq!(
            numbers(&rows),
            vec![
                "...", "2/2", "3/", "/3", "/4", "4/5", "...", "8/9", "/10", "9/11", "..."
            ]
        );
    }

    #[test]
    fn keeps_the_diff_without_context() {
        assert_eq!(
            numbers(&rows(&file(), None, 3)),
            vec!["@", "3/", "/3", "/4", "@", "/10"]
        );
    }

    #[test]
    fn puts_comments_outside_the_changes_in_line_order() {
        let old: Vec<String> = (1..=12).map(|n| format!("line {n}")).collect();
        let all: Vec<Row> = rows(&file(), Some(&old), old.len() + 1)
            .into_iter()
            .filter(|row| matches!(row, Row::Line { .. }))
            .collect();
        let comment = |id: &str, line: usize| Comment {
            id: id.into(),
            target: Target::Line {
                path: "a.txt".into(),
                start: line,
                end: line,
                old: false,
            },
            author: "me@example.com".into(),
            quote: Vec::new(),
            text: "x".into(),
        };
        let (first, middle) = (comment("c1", 1), comment("c2", 7));
        let shown = with_snippets(rows(&file(), None, 0), &all, &[&first, &middle]);
        assert_eq!(
            numbers(&shown),
            vec![
                "1/1", "2/2", "@", "3/", "/3", "/4", "4/5", "5/6", "6/7", "7/8", "8/9", "@", "/10"
            ]
        );
    }

    #[test]
    fn filters_a_line_range() {
        let rows = filter_lines(rows(&file(), None, 0), 9, 12);
        assert_eq!(numbers(&rows), vec!["@", "/10"]);
    }
}
