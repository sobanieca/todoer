//! `todoer pr ...`: submit, ls, merge, apply, close, summary, details, comment and review.

use crate::areas::{Areas, Filter};
use crate::args::{self, Args, Flag, option, switch};
use crate::colors;
use crate::commands::report;
use crate::commands::todos::print_removed_labels;
use crate::json::{self, Value};
use crate::pr_view::{self, DetailsOptions};
use crate::project::Project;
use crate::prs::{self, Destination, Pr, SubmitOptions};
use crate::review;
use crate::terminal;
use crate::todos::name_of;

struct Sub {
    name: &'static str,
    usage: &'static str,
    flags: &'static [Flag],
    run: fn(&Args) -> Result<(), String>,
}

const SUBMIT_FLAGS: &[Flag] = &[
    option("branch", Some('b')),
    option("name", Some('n')),
    option("message", Some('m')),
    option("todo", None),
    switch("update", None),
    switch("no-commit", None),
];
const LS_FLAGS: &[Flag] = &[switch("json", None)];
const NO_FLAGS: &[Flag] = &[];
const COMMIT_FLAGS: &[Flag] = &[switch("no-commit", None)];
const AREA_FLAGS: &[Flag] = &[option("area", Some('a')), option("exclude", Some('x'))];
const DETAILS_FLAGS: &[Flag] = &[
    option("area", Some('a')),
    option("exclude", Some('x')),
    option("context", None),
    option("lines", None),
];

const SUBS: &[Sub] = &[
    Sub {
        name: "submit",
        usage: "todoer pr submit --branch <branch> -n <name> [-m <text>] [--todo <name>]... [--update] [--no-commit]",
        flags: SUBMIT_FLAGS,
        run: submit,
    },
    Sub {
        name: "ls",
        usage: "todoer pr ls [--json]",
        flags: LS_FLAGS,
        run: ls,
    },
    Sub {
        name: "merge",
        usage: "todoer pr merge <pr> [--no-commit]",
        flags: COMMIT_FLAGS,
        run: merge,
    },
    Sub {
        name: "apply",
        usage: "todoer pr apply <pr>",
        flags: NO_FLAGS,
        run: apply,
    },
    Sub {
        name: "close",
        usage: "todoer pr close <pr> [--no-commit]",
        flags: COMMIT_FLAGS,
        run: close,
    },
    Sub {
        name: "summary",
        usage: "todoer pr summary <pr> [-a <area,...>] [-x <area,...>]",
        flags: AREA_FLAGS,
        run: summary,
    },
    Sub {
        name: "details",
        usage: "todoer pr details <pr> [file...] [--context N] [--lines 40-80] [-a <area,...>] [-x <area,...>]",
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
        name: "review",
        usage: "todoer pr review <pr> [-a <area,...>] [-x <area,...>]",
        flags: AREA_FLAGS,
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
    let todos: Vec<String> = args.values("todo").iter().map(|arg| name_of(arg)).collect();
    let result = prs::submit(
        &project,
        &SubmitOptions {
            branch: args.value("branch"),
            name: args.value("name"),
            message: args.value("message"),
            todos: &todos,
            update: args.has("update"),
            no_commit: args.has("no-commit"),
        },
    )?;
    let pr = &result.pr;
    let what = if result.created {
        "submitted"
    } else {
        "updated"
    };
    let commits = match result.commits {
        1 => "1 commit".to_string(),
        count => format!("{count} commits squashed to 1"),
    };
    println!(
        "{what} {}: {commits}, {}, into {}",
        pr.name,
        plural(result.files, "file"),
        result.into
    );
    match &result.destination {
        Destination::Checkout(target) => println!(
            "{} on {}, in the checkout {}",
            pr.rel_dir(),
            result.into,
            target.root.display()
        ),
        Destination::Branch(_) => println!("{} on {}", pr.rel_dir(), result.into),
    }
    if !pr.todos.is_empty() {
        println!("todos: {}", pr.todos.join(", "));
    }
    if result.kept.description {
        println!("kept the description of the old pull request - give -m to replace it");
    }
    if result.kept.todos {
        println!("kept the todos of the old pull request - give --todo to replace them");
    }
    if let Some(review) = result.cleared.filter(|review| !review.comments.is_empty()) {
        println!(
            "removed the review ({}) - git history keeps it",
            plural(review.comments.len(), "comment")
        );
    }
    if result.plan.commit {
        println!("committed on {}", result.into);
    } else if result.plan.others.is_empty() {
        println!(
            "not committed (--no-commit) - the pull request is in the working tree of {}",
            result.into
        );
    } else {
        crate::logger::warn(&format!(
            "not committed - the checkout of {} has other changes: {}. The pull request is in its working tree. Commit it yourself",
            result.into,
            result.plan.others.join(", ")
        ));
    }
    Ok(())
}

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

fn ls(args: &Args) -> Result<(), String> {
    args.expect_positional(0, 0, usage("ls"))?;
    let project = Project::find()?;
    let list = prs::list(&project)?;
    if args.has("json") {
        let items = list
            .iter()
            .map(|pr| {
                Value::object(vec![
                    ("name", Value::str(&pr.name)),
                    ("path", Value::str(&pr.rel_dir())),
                    ("author", Value::str(&pr.author)),
                    ("comments", Value::Number(pr.review().comments.len() as f64)),
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
    let name_width = list.iter().map(|pr| pr.name.len()).max().unwrap_or(0);
    for pr in &list {
        let comments = pr.review().comments.len();
        let review = if comments == 0 {
            colors::gray("no comments")
        } else {
            colors::yellow(&plural(comments, "comment"))
        };
        println!(
            "{}  {}  {}",
            colors::pad(&colors::bold(&pr.name), name_width),
            colors::pad(&review, 11),
            pr.author
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
    let merged = prs::merge(&project, &pr, args.has("no-commit"))?;
    let branch = project.branch_name()?;
    if merged.applied {
        println!("merged {} into {branch}", pr.name);
    } else {
        println!(
            "merged {} into {branch}: the changes were already on {branch}",
            pr.name
        );
    }
    for name in &merged.done {
        println!("done: {name}");
    }
    for name in &merged.changed {
        println!("updated dependsOn in {name}");
    }
    print_removed_labels(&merged.removed_labels);
    report(&merged.plan);
    Ok(())
}

fn apply(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, usage("apply"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let applied = prs::apply(&project, &pr)?;
    let branch = project.branch_name()?;
    if applied {
        println!(
            "applied {} to the working tree of {branch} - not committed",
            pr.name
        );
    } else {
        println!("the changes of {} are already on {branch}", pr.name);
    }
    println!(
        "to update the pull request, commit your changes, then run: todoer pr submit --branch <the branch that the pull request goes into> -n {} --update -m \"<what you changed>\"",
        pr.name
    );
    Ok(())
}

fn close(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, usage("close"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let plan = prs::close(&project, &pr, args.has("no-commit"))?;
    println!("closed {}", pr.name);
    if !pr.todos.is_empty() {
        println!(
            "the todos stay: {} - run todoer unclaim <name> to free a todo",
            pr.todos.join(", ")
        );
    }
    report(&plan);
    Ok(())
}

fn summary(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, usage("summary"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let areas = Areas::load(&project.root);
    let filter = area_filter(&areas, args)?;
    print!("{}", pr_view::summary(&pr, &filter));
    Ok(())
}

fn area_filter<'a>(areas: &'a Areas, args: &Args) -> Result<Filter<'a>, String> {
    Filter::new(areas, &args.values("area"), &args.values("exclude"))
}

fn details_text(
    project: &Project,
    pr: &Pr,
    args: &Args,
    paths: Vec<String>,
    filter: &Filter,
) -> Result<String, String> {
    let lines = args.value("lines").map(pr_view::parse_lines).transpose()?;
    let options = DetailsOptions {
        paths,
        context: args.number("context")?.unwrap_or(0),
        lines,
        filter,
    };
    pr_view::details(project, pr, &options)
}

fn details(args: &Args) -> Result<(), String> {
    args.expect_positional(1, usize::MAX, usage("details"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let paths = args.positional[1..]
        .iter()
        .map(|path| project.relative(path).unwrap_or_else(|_| path.clone()))
        .collect();
    let areas = Areas::load(&project.root);
    let filter = area_filter(&areas, args)?;
    let text = details_text(&project, &pr, args, paths, &filter)?;
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
        "added the comment {} on {} in {}",
        comment.id,
        if target.is_some() {
            comment.target.format()
        } else {
            "the pull request".into()
        },
        pr.name
    );
    println!(
        "the review is not committed - when it is done, run: git add {path} && git commit -m \"Review {}\" -- {path}",
        pr.name,
        path = pr.rel(prs::REVIEW)
    );
    Ok(())
}

fn review_command(args: &Args) -> Result<(), String> {
    args.expect_positional(1, 1, usage("review"))?;
    let project = Project::find()?;
    let pr = prs::resolve(&project, first(args))?;
    let areas = Areas::load(&project.root);
    let filter = area_filter(&areas, args)?;
    let mut text = pr_view::summary(&pr, &filter);
    text.push('\n');
    text.push_str(&details_text(&project, &pr, args, Vec::new(), &filter)?);
    terminal::page(&text);
    Ok(())
}
