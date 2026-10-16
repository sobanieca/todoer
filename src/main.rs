mod areas;
mod args;
mod colors;
mod commands;
mod diff;
mod frontmatter;
mod git;
mod json;
mod logger;
mod pr_view;
mod project;
mod prs;
mod review;
mod rules;
mod terminal;
mod todos;
mod update;

use args::Flag;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP: &str = include_str!("help.txt");

type Run = fn(&args::Args) -> Result<(), String>;

const COMMANDS: &[(&str, &[Flag], Run)] = &[
    ("ls", commands::todos::LS_FLAGS, commands::todos::ls),
    ("add", commands::todos::ADD_FLAGS, commands::todos::add),
    ("done", &[], commands::todos::done),
    ("mv", &[], commands::todos::mv),
    ("check", &[], commands::todos::check),
    (
        "claim",
        commands::todos::CLAIM_FLAGS,
        commands::todos::claim,
    ),
    (
        "unclaim",
        commands::todos::UNCLAIM_FLAGS,
        commands::todos::unclaim,
    ),
    ("dep", commands::deps::FLAGS, commands::deps::run),
    ("label", &[], commands::labels::label),
    (
        "labels",
        commands::labels::LABELS_FLAGS,
        commands::labels::labels,
    ),
    (
        "progress",
        commands::labels::PROGRESS_FLAGS,
        commands::labels::progress,
    ),
    ("skill", commands::skill::FLAGS, commands::skill::run),
];

fn dispatch(raw: &[String]) -> Result<(), String> {
    let Some(name) = raw.first().map(String::as_str) else {
        print!("{HELP}");
        return Ok(());
    };
    match name {
        "help" | "--help" | "-h" => {
            print!("{HELP}");
            Ok(())
        }
        "version" | "--version" | "-V" => {
            println!("{VERSION}");
            Ok(())
        }
        "update" => update::execute(),
        "pr" => commands::pr::run(&raw[1..]),
        _ => {
            let (_, flags, run) = COMMANDS
                .iter()
                .find(|(command, _, _)| *command == name)
                .ok_or_else(|| format!("unknown command \"{name}\" - run todoer help"))?;
            let parsed = args::parse(&raw[1..], flags, name)?;
            if parsed.has("help") {
                print!("{HELP}");
                return Ok(());
            }
            run(&parsed)
        }
    }
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    colors::init(terminal::stdout_is_terminal());
    let is_update = matches!(
        raw.first().map(String::as_str),
        Some("update" | "version" | "--version" | "-V")
    );
    let check = if is_update {
        None
    } else {
        update::start_background_check()
    };
    let code = match dispatch(&raw) {
        Ok(()) => 0,
        Err(message) => {
            logger::error(&message);
            1
        }
    };
    update::notify(check);
    std::process::exit(code);
}
