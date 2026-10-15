//! `todoer pr ...`: submit, ls, merge, close, summary, details, comment, approve, request-changes
//! and review.

use crate::args::{self, Args, Flag, option, switch};
use crate::colors;
use crate::json::{self, Value};
use crate::owners::{Filter, Owners};
use crate::pr_view::{self, DetailsOptions};
use crate::project::Project;
use crate::prs::{self, Pr};
use crate::review;
use crate::terminal;

const OWNER_FLAGS: [Flag; 3] = [
    option("owner", None),
    switch("mine", None),
    switch("unowned", None),
];

struct Sub {
    name: &'static str,
    usage: &'static str,
    flags: &'static [Flag],
    run: fn(&Args) -> Result<(), String>,
}

const SUBMIT_FLAGS: &[Flag] = &[
    option("title", Some('t')),
    option("message", Some('m')),
    option("todo", None),
    switch("update", None),
];
const LS_FLAGS: &[Flag] = &[switch("mine", None), switch("json", None)];
const NO_FLAGS: &[Flag] = &[];
const SUMMARY_FLAGS: &[Flag] = &[OWNER_FLAGS[0], OWNER_FLAGS[1], OWNER_FLAGS[2]];
const DETAILS_FLAGS: &[Flag] = &[
    OWNER_FLAGS[0],
    OWNER_FLAGS[1],
    OWNER_FLAGS[2],
    option("context", None),
    option("lines", None),
];
const VERDICT_FLAGS: &[Flag] = &[option("message", Some('m'))];
const REVIEW_FLAGS: &[Flag] = &[OWNER_FLAGS[0], OWNER_FLAGS[1], OWNER_FLAGS[2]];

const SUBS: &[Sub] = &[
    Sub {
        name: "submit",
        usage: "todoer pr submit [-t <title>] [-m <text>] [--todo <path>]... [--update]",
        flags: SUBMIT_FLAGS,
        run: submit,
    },
    Sub {
        name: "ls",
        usage: "todoer pr ls [--mine] [--json]",
        flags: LS_FLAGS,
        run: ls,
    },
    Sub {
        name: "merge",
        usage: "todoer pr merge <pr>",
        flags: NO_FLAGS,
        run: merge,
    },
    Sub {
        name: "close",
        usage: "todoer pr close <pr>",
        flags: NO_FLAGS,
        run: close,
    },
    Sub {
        name: "summary",
        usage: "todoer pr summary <pr> [--owner <team|email>] [--mine] [--unowned]",
        flags: SUMMARY_FLAGS,
        run: summary,
    },
    Sub {
        name: "details",
        usage: "todoer pr details <pr> [file...] [--context N] [--lines 40-80] [--owner <team|email>] [--mine] [--unowned]",
        flags: DETAILS_FLAGS,
        run: details,
    },
    Sub {
        name: "comment",
        usage: "todoer pr comment <pr> [target] [\"<text>\"]",
        flags: NO_FLAGS,
        run: comment,
    },
    Sub {
        name: "approve",
        usage: "todoer pr approve <pr> [-m <text>]",
        flags: VERDICT_FLAGS,
        run: approve,
    },
    Sub {
        name: "request-changes",
        usage: "todoer pr request-changes <pr> [-m <text>]",
        flags: VERDICT_FLAGS,
        run: request_changes,
    },
    Sub {
        name: "review",
        usage: "todoer pr review <pr> [--owner <team|email>] [--mine] [--unowned]",
        flags: REVIEW_FLAGS,
        run: review_command,
    },
];

pub fn run(raw: &[String]) -> Result<(), String> {
    let Some(name) = raw.first() else {
        return Err(usage_all());
    };
    if name == "--help" || name == "-h" || name == "help" {
        println!("{}", usage_all());
        return Ok(());
    }
    let sub = SUBS
        .iter()
        .find(|sub| sub.name == name)
        .ok_or_else(|| format!("unknown command \"todoer pr {name}\"\n{}", usage_all()))?;
    let parsed = args::parse(&raw[1..], sub.flags, &format!("pr {name}"))?;
    if parsed.has("help") {
        println!("use: {}", sub.usage);
        return Ok(());
    }
    (sub.run)(&parsed)
}

fn usage(name: &str) -> &'static str {
    SUBS.iter()
        .find(|sub| sub.name == name)
        .map(|sub| sub.usage)
        .unwrap_or("todoer pr help")
}

fn usage_all() -> String {
    let mut out = String::from("use:\n");
    for sub in SUBS {
        out.push_str(&format!("  {}\n", sub.usage));
    }
    out.trim_end().to_string()
}

fn submit(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 0, usage("submit"))?;
    let project = Project::find()?;
    let todos = args
        .values("todo")
        .iter()
        .map(|path| project.relative(path))
        .collect::<Result<Vec<_>, _>>()?;
    let result = prs::submit(
        &project,
        args.value("title"),
        args.value("message"),
        &todos,
        args.has("update"),
    )?;
    let pr = &result.pr;
    let what = if result.created {
        "submitted"
    } else {
        "updated"
    };
    println!(
        "{what} {}: {}, {}",
        pr.slug,
        plural(result.commits, "commit"),
        plural(result.files, "file")
    );
    println!("{}", pr.rel_dir());
    if !pr.todos.is_empty() {
        println!("todos: {}", pr.todos.join(", "));
    }
    if let Some(review) = result
        .cleared
        .filter(|review| !review.comments.is_empty() || review.verdict != "pending")
    {
        println!(
            "removed the review ({}, {}) - git history keeps it",
            review.verdict,
            plural(review.comments.len(), "comment")
        );
    }
    Ok(())
}

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

fn ls(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 0, usage("ls"))?;
    let project = Project::find()?;
    let mut list = prs::list(&project)?;
    if args.has("mine") {
        let owners = Owners::load(&project.root);
        let filter = Filter::Mine(owners.identities(&project.me()?));
        list.retain(|pr| {
            pr.files()
                .iter()
                .any(|file| filter.keep(&owners.of_file(file)))
        });
    }
    if args.has("json") {
        let items = list
            .iter()
            .map(|pr| {
                Value::object(vec![
                    ("slug", Value::str(&pr.slug)),
                    ("path", Value::str(&pr.rel_dir())),
                    ("title", Value::str(&pr.title)),
                    ("author", Value::str(&pr.author)),
                    ("branch", Value::str(&pr.branch)),
                    ("verdict", Value::str(&pr.verdict(&project))),
                    ("todos", Value::strings(&pr.todos)),
                ])
            })
            .collect();
        println!("{}", json::stringify(&Value::Array(items)));
        return Ok(());
    }
    if list.is_empty() {
        println!("no open pull requests");
        return Ok(());
    }
    let slug_width = list.iter().map(|pr| pr.slug.len()).max().unwrap_or(0);
    let author_width = list.iter().map(|pr| pr.author.len()).max().unwrap_or(0);
    for pr in &list {
        let verdict = pr.verdict(&project);
        println!(
            "{}  {}  {}  {}",
            colors::pad(&colors::bold(&pr.slug), slug_width),
            colors::pad(&pr_view::verdict_color(&verdict), 17),
            colors::pad(&pr.author, author_width),
            pr.title
        );
    }
    Ok(())
}

fn first(args: &Args) -> &str {
    &args.positional[0]
}

fn merge(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, usage("merge"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let merged = prs::merge(&project, &pr)?;
    println!(
        "merged {} on {}: {}",
        pr.slug,
        project.main_branch,
        plural(merged.commits, "commit")
    );
    for path in &merged.done {
        println!("done: {path}");
    }
    for warning in &merged.warnings {
        crate::logger::warn(warning);
    }
    Ok(())
}

fn close(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, usage("close"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    prs::close(&project, &pr)?;
    println!("closed {}", pr.slug);
    if !pr.todos.is_empty() {
        println!(
            "the todos keep their status: {} - run todoer unclaim <path> to free a todo",
            pr.todos.join(", ")
        );
    }
    Ok(())
}

fn summary(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, usage("summary"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let owners = Owners::load(&project.root);
    let filter = Filter::from_args(args, &project, &owners)?;
    print!("{}", pr_view::summary(&project, &pr, &owners, &filter));
    Ok(())
}

fn details_text(
    project: &Project,
    pr: &Pr,
    args: &Args,
    paths: Vec<String>,
    owners: &Owners,
) -> Result<String, String> {
    let filter = Filter::from_args(args, project, owners)?;
    let lines = args.value("lines").map(pr_view::parse_lines).transpose()?;
    let options = DetailsOptions {
        paths,
        context: args.number("context")?.unwrap_or(0),
        lines,
        filter: &filter,
    };
    pr_view::details(project, pr, owners, &options)
}

fn details(args: &Args) -> Result<(), String> {
    args.expect_positional(1, usize::MAX, usage("details"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let paths = args.positional[1..]
        .iter()
        .map(|path| project.relative(path).unwrap_or_else(|_| path.clone()))
        .collect();
    let owners = Owners::load(&project.root);
    let text = details_text(&project, &pr, args, paths, &owners)?;
    terminal::page(&text);
    Ok(())
}

fn read_text(given: Option<&String>) -> Result<String, String> {
    if let Some(text) = given {
        return Ok(text.clone());
    }
    let text = terminal::read_stdin_if_piped()?;
    if text.trim().is_empty() {
        return Err("a comment needs text - give it as an argument, or on stdin".into());
    }
    Ok(text)
}

fn looks_like_target(project: &Project, pr: &Pr, text: &str) -> bool {
    !text.contains(char::is_whitespace)
        && review::parse_target(text).is_ok_and(|target| {
            review::check_target(&target, &pr.files(), &pr.tree(project)).is_ok()
        })
}

fn comment(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 3, usage("comment"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let rest = &args.positional[1..];
    let (target, text) = match rest {
        [] => (None, None),
        [one] if looks_like_target(&project, &pr, one) => (Some(one.as_str()), None),
        [one] => (None, Some(one)),
        [target, text] => (Some(target.as_str()), Some(text)),
        _ => return Err(format!("use: {}", usage("comment"))),
    };
    let text = read_text(text)?;
    let comment = prs::add_comment(&project, &pr, target, &text)?;
    println!(
        "added the draft comment {} on {} in {}",
        comment.id,
        if target.is_some() {
            comment.target.format()
        } else {
            "the pull request".into()
        },
        pr.slug
    );
    println!("run todoer pr approve or todoer pr request-changes to send the review");
    Ok(())
}

fn verdict(args: &Args, verdict: &str, usage: &str) -> Result<(), String> {
    args.expect_positional(1, 1, usage)?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let review = prs::set_verdict(&project, &pr, verdict, args.value("message"))?;
    println!(
        "{} {}: {}",
        verdict.replace('-', " "),
        pr.slug,
        plural(review.comments.len(), "comment")
    );
    Ok(())
}

fn approve(args: &Args) -> Result<(), String> {
    verdict(args, "approved", usage("approve"))
}

fn request_changes(args: &Args) -> Result<(), String> {
    verdict(args, "changes-requested", usage("request-changes"))
}

fn review_command(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, usage("review"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let owners = Owners::load(&project.root);
    let filter = Filter::from_args(args, &project, &owners)?;
    let mut text = pr_view::summary(&project, &pr, &owners, &filter);
    text.push('\n');
    text.push_str(&details_text(&project, &pr, args, Vec::new(), &owners)?);
    terminal::page(&text);
    Ok(())
}
