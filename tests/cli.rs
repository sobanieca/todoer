//! End to end tests: run the `todoer` binary in a temporary git repository and compare the output
//! with the snapshots in `tests/snapshots`.

mod support;

use support::Session;

fn commit(s: &Session, message: &str) {
    s.quiet(&format!("git add -A && git commit -q -m \"{message}\""));
}

fn worktree(s: &Session, branch: &str) -> std::path::PathBuf {
    s.quiet(&format!("git worktree add -q -b {branch} ../{branch} main"));
    s.path(branch)
}

#[test]
fn help_and_errors() {
    let mut s = Session::new("help");
    s.run("todoer help");
    s.run("todoer nope");
    s.run("todoer ls --nope");
    s.run("todoer pr");
    s.run("todoer pr nope");
    s.run("todoer pr details --help");
    s.run("todoer done");
    s.run("cd .. && todoer ls");
    s.assert_snapshot("help_and_errors");
}

#[test]
fn todos() {
    let mut s = Session::new("todos");
    s.run("todoer add port-cli-parser -l rust-port 'Parse the arguments.'");
    s.run("todoer add port-config -l rust-port -l config -d port-cli-parser");
    s.run("echo 'From stdin.' | todoer add release-v1 -d .todoer/todos/port-config.md");
    s.run("todoer add port-cli-parser");
    s.run("todoer add");
    s.run("todoer add Fix_Login");
    s.run("todoer add x -l Bad");
    s.run("todoer add x -d missing");
    s.note_file(".todoer/todos/port-config.md");
    s.note_file(".todoer/todos/release-v1.md");
    s.write(".todoer/todos/notes.txt", "Not a todo.\n");
    s.write(".todoer/todos/drafts/old.md", "Not a todo.\n");
    s.run("todoer ls");
    s.run("todoer ls --available");
    s.run("todoer ls --label config --json");
    s.run("cd .todoer/todos && todoer claim port-cli-parser.md");
    s.run("todoer ls");
    s.run("todoer dep show port-config");
    s.run("todoer dep add port-cli-parser release-v1");
    s.run("todoer dep add release-v1 release-v1");
    s.run("todoer dep add release-v1 port-cli-parser");
    s.run("todoer dep show port-cli-parser");
    s.run("todoer dep rm release-v1 port-cli-parser");
    s.run("todoer dep rm release-v1 port-cli-parser");
    s.run("todoer label add release-v1 release");
    s.run("todoer label add release-v1 rust-port");
    s.run("todoer label rm release-v1 release");
    s.run("todoer label add release-v1 Release");
    s.run("todoer labels");
    s.run("todoer progress");
    s.run("todoer progress rust-port");
    s.run("todoer progress rust-port --json");
    s.run("todoer progress missing");
    s.run("todoer labels rm config");
    s.run("todoer labels rm config");
    s.run("todoer labels rm");
    s.run("todoer label add port-cli-parser core");
    s.run("todoer labels mv rust-port core");
    s.note_file(".todoer/todos/port-cli-parser.md");
    s.run("todoer labels mv core Core");
    s.run("todoer labels mv core core");
    s.run("todoer labels mv missing other");
    s.run("todoer labels mv core rust-port");
    s.run("todoer labels");
    s.run("todoer mv port-config config-file");
    s.note_file(".todoer/todos/release-v1.md");
    s.run("todoer mv config-file Config_File");
    s.run("todoer mv missing other");
    s.run("todoer done port-cli-parser");
    s.note_file(".todoer/todos/config-file.md");
    s.run("todoer ls");
    s.run("todoer check");
    s.write(
        ".todoer/todos/Bad.md",
        "---\nassignee: bob\nlabels: [Rust]\ndependsOn: [gone, Bad]\n---\n",
    );
    s.run("todoer check");
    s.run("todoer ls");
    s.run("git status --short && git log --format=%s");
    s.assert_snapshot("todos");
}

#[test]
fn label_dependencies() {
    let mut s = Session::new("label-deps");
    s.run("todoer add new-login-page -D auth-backend-refactor");
    s.run("todoer add move-tokens -l auth-backend-refactor");
    s.run("todoer add move-sessions -l auth-backend-refactor");
    s.run("todoer add new-login-page -D auth-backend-refactor -l ui 'Build the page.'");
    s.note_file(".todoer/todos/new-login-page.md");
    s.run("todoer add login-help -D ui -D Bad");
    s.run("todoer add login-help -d new-login-page -l auth-backend-refactor");
    s.run("todoer ls");
    s.run("todoer ls --label ui --json");
    s.run("todoer dep show new-login-page");
    s.run("todoer dep show move-tokens");
    s.run("todoer claim new-login-page");
    s.run("todoer label add new-login-page auth-backend-refactor");
    s.run("todoer dep add move-tokens -l ui");
    s.run("todoer dep add move-tokens -l missing");
    s.run("todoer add release -d move-tokens");
    s.run("todoer dep add release -l ui -l auth-backend-refactor");
    s.run("todoer dep rm release -l ui");
    s.run("todoer dep rm release -l ui");
    s.run("todoer dep show release -l ui");
    s.run("todoer labels mv auth-backend-refactor auth");
    s.note_file(".todoer/todos/new-login-page.md");
    s.run("todoer progress ui");
    s.run("todoer done move-tokens");
    s.run("todoer ls");
    s.run("todoer label rm move-sessions auth");
    s.note_file(".todoer/todos/new-login-page.md");
    s.run("todoer ls");
    s.write(
        ".todoer/todos/hand-edit.md",
        "---\ndependsOnLabels: [gone, Bad]\n---\n",
    );
    s.run("todoer check");
    s.assert_snapshot("label_dependencies");
}

#[test]
fn label_rules() {
    let mut s = Session::new("rules");
    s.run("todoer add fix-login -l auth");
    s.run("todoer labels rule auth 'Add this label to a todo that changes the login.'");
    s.run("printf 'Add it to a todo that changes the docs.\\nNot to a typo fix.\\n' | todoer labels rule docs");
    s.note_file(".todoer/labels/docs.md");
    s.run("todoer labels rule auth --rm extra");
    s.run("todoer labels rule Auth 'A rule.'");
    s.run("todoer labels rule ui < /dev/null");
    s.run("todoer labels rule ui --rm");
    s.run("todoer labels --rm");
    s.run("todoer add login-page");
    s.run("todoer labels");
    s.run("todoer labels --json");
    s.run("todoer labels mv auth docs");
    s.run("todoer labels mv auth security");
    s.run("todoer labels mv docs guides");
    s.run("todoer labels mv missing other");
    s.run("todoer labels");
    s.run("todoer labels rule security --rm");
    s.run("todoer labels rm guides");
    s.run("todoer labels rm security");
    s.run("todoer labels");
    s.run("todoer check");
    s.write(".todoer/labels/Bad.md", "A rule.\n");
    s.write(".todoer/labels/empty.md", "\n");
    s.write(".todoer/labels/notes.txt", "Not a rule.\n");
    s.run("todoer check");
    s.run("todoer labels");
    s.run("git status --short");
    s.assert_snapshot("label_rules");
}

#[test]
fn claims() {
    let mut s = Session::new("claims");
    s.run("todoer add first -l core");
    s.run("todoer add second -l core -d first");
    s.run("todoer add third -d second");
    commit(&s, "todos");
    s.run("todoer claim");
    s.run("todoer claim second");
    s.run("todoer claim first");
    s.run("todoer claim first");
    s.run("git log --format=%s -1 && git status --short");
    s.note_file(".todoer/todos/first.md");
    s.run("git config user.email other@example.com && todoer claim first");
    s.run("todoer claim --label core");
    s.run("todoer claim third");
    s.run("todoer claim third second");
    s.run("git log --format=%s -1 && git status --short");
    s.run("git config user.email me@example.com && todoer ls --mine");
    s.run("todoer claim --label nope");
    s.run("todoer unclaim first");
    s.run("todoer unclaim first");
    s.run("git log --format=%s -1 && git status --short");
    s.note_file(".todoer/todos/first.md");
    s.write("notes.txt", "notes\n");
    commit(&s, "notes");
    s.quiet("echo more >> notes.txt");
    s.run("todoer claim first && git log --format=%s -1 && git status --short");
    s.quiet("git checkout -q -- notes.txt");
    s.run("todoer unclaim first --no-commit && git status --short");
    s.quiet("git checkout -q -- .todoer");
    let feature = worktree(&s, "feature");
    s.run_in("todoer ls", &feature);
    s.run_in(
        "todoer claim first && git log --format=%s -1 && git status --short",
        &feature,
    );
    s.assert_snapshot("claims");
}

#[test]
fn todos_of_many_projects() {
    let mut s = Session::new("projects");
    s.quiet("todoer add fix-login -l auth");
    s.quiet("mkdir ../other && cd ../other && git init -q && git commit -q --allow-empty -m init && git checkout -q -b feat/some-feature && todoer add write-docs && todoer claim write-docs");
    s.quiet("mkdir ../empty && cd ../empty && git init -q");
    s.quiet("mkdir -p ../work/api && cd ../work/api && git init -q && git commit -q --allow-empty -m init && todoer add add-rate-limit");
    s.quiet("cd ../other && git worktree add -q ../other-docs");
    s.run("cd .. && todoer ls");
    s.run("cd .. && todoer ls --mine");
    s.run("cd .. && todoer ls --label auth --json");
    s.run("cd ../home && todoer ls");
    s.assert_snapshot("todos_of_many_projects");
}

fn setup_pr(s: &Session) -> std::path::PathBuf {
    s.write(
        "src/app.rs",
        "fn main() {\n    let a = 1;\n    let b = 2;\n    println!(\"{}\", a + b);\n}\n",
    );
    s.write("src/old.rs", "pub fn old() {}\n");
    s.write(
        "docs/guide.md",
        "# Guide\n\nline 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\n",
    );
    s.quiet("todoer add fix-the-sum");
    commit(s, "start");
    s.quiet("todoer claim fix-the-sum");
    let work = worktree(s, "fix-sum");
    s.write(
        "../fix-sum/src/app.rs",
        "fn main() {\n    let a = 1;\n    let b = 3;\n    println!(\"{}\", a + b);\n}\n",
    );
    s.write("../fix-sum/docs/guide.md", "# Guide\n\nline 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n");
    s.quiet("cd ../fix-sum && git mv src/old.rs src/lib.rs && git add -A && git commit -q -m 'Use three for b'");
    work
}

const PR: &str = ".todoer/prs/use-three-for-b";

#[test]
fn pull_requests() {
    let mut s = Session::new("prs");
    s.run("todoer pr ls");
    let work = setup_pr(&s);
    s.run_in("todoer pr submit", &work);
    s.run_in("todoer pr submit --branch nope", &work);
    s.run_in("todoer pr submit --branch fix-sum", &work);
    s.run_in(
        "todoer pr submit --branch main -m 'b is three now.' --todo fix-the-sum",
        &work,
    );
    s.run("git log --format=%s -3 && git status --short && ls -a .todoer/prs/use-three-for-b");
    s.note_file(&format!("{PR}/pr.md"));
    s.run_in("todoer pr ls", &work);
    s.run("todoer pr ls");
    s.run("todoer pr ls --json");
    s.run("todoer pr summary use-three-for-b");
    s.run("todoer pr details .todoer/prs/use-three-for-b/");
    s.run("todoer pr details use-three-for-b docs/guide.md --context 2");
    s.run("todoer pr details use-three-for-b src/app.rs --context 1 --lines 3-3");
    s.run("todoer pr details use-three-for-b nope.rs");
    let comments = [
        "src/app.rs:3 'Use a constant.'",
        "src/app.rs:2-4 'Format this.'",
        "src/app.rs:old:3 'Why two?'",
        "src/lib.rs 'Name it math.rs.'",
        "src/ 'Move under lib/.'",
        "'Add a test.'",
        "docs/guide.md:3 'Outside the changes.'",
        ".todoer/todos/fix-the-sum.md:1 'Not in the pull request.'",
        "src/app.rs:40 'x'",
        "src/nope.rs 'x'",
        "../x 'x'",
        "src/app.rs:1",
    ];
    for comment in comments {
        s.run(&format!("todoer pr comment use-three-for-b {comment}"));
    }
    s.run("echo 'From stdin.' | todoer pr comment use-three-for-b docs/guide.md:12");
    s.run("todoer pr summary use-three-for-b");
    s.run("todoer pr details use-three-for-b");
    s.run("git status --short");
    s.note_file(&format!("{PR}/review.md"));
    s.quiet(&format!(
        "git add {PR}/review.md && git commit -q -m 'Review use-three-for-b' -- {PR}/review.md"
    ));
    s.run("todoer pr ls && todoer progress");
    s.write("../fix-sum/src/app.rs", "fn main() {\n    let a = 1;\n    let b = B;\n    println!(\"{}\", a + b);\n}\n\nconst B: i32 = 3;\n");
    s.quiet("cd ../fix-sum && git commit -q -am 'Add a constant'");
    s.run_in("todoer pr submit --branch main -m 'A constant.'", &work);
    s.run_in(
        "todoer pr submit --branch main -m 'A constant.' --update",
        &work,
    );
    s.run("git log --format=%s -1 && git status --short && ls .todoer/prs/use-three-for-b");
    s.note_file(&format!("{PR}/pr.md"));
    s.run("todoer pr summary use-three-for-b");
    s.run("todoer pr comment use-three-for-b src/app.rs:7 'Why i32?'");
    s.run("todoer pr comment use-three-for-b docs/guide.md 'Say what the guide is for.'");
    s.run("todoer pr merge use-three-for-b");
    s.quiet("git add -A && git commit -q -m 'Review again'");
    s.write("../fix-sum/docs/guide.md", "# Guide\n\nWhat the guide is for.\n\nline 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\n");
    s.quiet("cd ../fix-sum && git commit -q -am 'Explain the guide'");
    s.run_in("todoer pr submit --branch main --update", &work);
    s.run("todoer pr comment");
    s.run("todoer pr merge use-three-for-b");
    s.run("git log --format=%s -3 && git status --short && ls .todoer && cat src/app.rs");
    s.run("git show --stat --format='%an <%ae>%n%B' HEAD");
    s.run("todoer pr ls && todoer ls");
    s.assert_snapshot("pull_requests");
}

#[test]
fn merging_and_closing() {
    let mut s = Session::new("merge");
    s.write("notes.txt", "notes\n");
    commit(&s, "notes");
    let work = setup_pr(&s);
    s.run_in("todoer pr submit --branch main -n 'First try'", &work);
    s.run_in("todoer pr submit --branch main -n first-try", &work);
    s.run("todoer pr close first-try");
    s.run("git log --format=%s -1 && ls .todoer");
    s.quiet("echo more >> ../fix-sum/notes.txt");
    s.run_in(
        "todoer pr submit --branch main -n second-try --todo fix-the-sum",
        &work,
    );
    s.quiet("cd ../fix-sum && git checkout -q -- notes.txt");
    s.run_in(
        "todoer pr submit --branch main -n second-try --todo fix-the-sum",
        &work,
    );
    s.run_in("todoer pr submit --branch main -n third-try", &work);
    s.quiet("cd ../fix-sum && git switch -q -c fix-sum-2");
    s.run_in("todoer pr submit --branch main -n second-try", &work);
    s.run_in(
        "todoer pr submit --branch main -n second-try --update -m 'From a new branch.'",
        &work,
    );
    s.run("todoer pr ls && git log --format=%s -1 && git status --short");
    s.quiet("git tag picked");
    s.quiet("echo more >> notes.txt");
    s.run("todoer pr merge second-try");
    s.run("git log --format=%s -1 && git status --short");
    s.quiet("git reset -q --hard picked");
    s.run("todoer pr merge second-try --no-commit");
    s.run("git log --format=%s -1 && git status --short && cat src/app.rs");
    s.quiet("git reset -q --hard picked && git clean -qfd src");
    s.write("src/app.rs", "fn main() {}\n");
    commit(&s, "change app on main");
    s.run("todoer pr merge second-try");
    s.run("todoer pr merge second-try --no-commit");
    s.run("git log --format=%s -1 && git status --short");
    s.quiet("git reset -q --hard picked");
    s.run("todoer pr comment second-try 'Add a test.'");
    s.run("todoer pr merge second-try");
    s.run("todoer pr close second-try --no-commit && git status --short");
    s.quiet("git reset -q --hard picked");
    s.run("todoer pr close second-try && git log --format=%s -1 && todoer pr ls");
    s.assert_snapshot("merging_and_closing");
}

#[test]
fn submitting_into_other_branches() {
    let mut s = Session::new("into");
    let work = setup_pr(&s);
    s.quiet("git switch -q -c feature");
    s.run_in("todoer pr submit --branch main --no-commit", &work);
    s.run_in("todoer pr submit --branch main --todo fix-the-sum", &work);
    s.run("git log --format=%s -2 main && git status --short && todoer pr ls");
    s.run("git switch -q main && todoer pr ls && todoer pr merge use-three-for-b");
    s.run("git log --format=%s -3 && git status --short && todoer ls");
    s.quiet("git switch -q feature && echo more >> src/old.rs");
    s.run_in("todoer pr submit --branch feature -n into-feature", &work);
    s.run("git log --format=%s -1 && git status --short");
    s.quiet("git checkout -q -- src/old.rs && rm -r .todoer/prs");
    s.run_in(
        "todoer pr submit --branch feature -n into-feature --no-commit",
        &work,
    );
    s.run("git status --short");
    s.assert_snapshot("submitting_into_other_branches");
}

#[test]
fn code_areas() {
    let mut s = Session::new("areas");
    s.write(
        ".todoer/areas",
        "# the last area that matches wins\nrust: *.rs\nlegacy: src/old.rs\napp: /src/app.rs /src/main.rs\nguides: guide.md\n",
    );
    commit(&s, "areas");
    let work = setup_pr(&s);
    s.run_in("todoer pr submit --branch main", &work);
    let runs = [
        "todoer pr summary use-three-for-b",
        "todoer pr summary use-three-for-b -a legacy,app",
        "todoer pr details use-three-for-b --area app",
        "todoer pr details use-three-for-b -a legacy -a rust",
        "todoer pr summary use-three-for-b -x app",
        "todoer pr details use-three-for-b -x rust,legacy,app",
        "todoer pr details use-three-for-b -a app -x app",
        "todoer pr summary use-three-for-b -a nope",
    ];
    for command in runs {
        s.run(command);
    }
    s.assert_snapshot("code_areas");
}

#[test]
fn skill() {
    let mut s = Session::new("skill");
    s.run("todoer skill | head -5");
    s.run("todoer skill --init");
    s.run("todoer skill --init");
    s.run(
        "ls .claude/skills/todoer .agents/skills/todoer && head -3 .claude/skills/todoer/SKILL.md",
    );
    s.assert_snapshot("skill");
}
