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
    s.quiet("mkdir -p todo/rust");
    s.run(
        "cd todo/rust && todoer add -t 'Port the CLI parser' -l rust-port 'Parse the arguments.'",
    );
    s.run("todoer add -t 'Port: the config' -p todo/rust -l rust-port -l config -d todo/rust/port-the-cli-parser.md");
    s.run(
        "echo 'From stdin.' | todoer add -t 'Release v1' -p todo -d todo/rust/port-the-config.md",
    );
    s.run("todoer add -t 'Port the CLI parser' -p todo/rust");
    s.run("todoer add 'no title'");
    s.run("todoer add -t x -l Bad");
    s.run("todoer add -t x -d todo/missing.md");
    s.note_file("todo/rust/port-the-config.md");
    s.note_file("todo/release-v1.md");
    s.write("README.md", "# Not a todo\n");
    s.write("docs/plan.md", "---\ntitle: a plan\n---\n\nNo status.\n");
    s.run("todoer ls");
    s.run("todoer ls --ready");
    s.run("todoer ls --label config --json");
    s.run("cd todo && todoer done rust/port-the-cli-parser-2.md");
    s.run("todoer ls --all");
    s.run("todoer dep show todo/rust/port-the-config.md");
    s.run("todoer dep add todo/rust/port-the-cli-parser.md todo/release-v1.md");
    s.run("todoer dep add todo/release-v1.md todo/release-v1.md");
    s.run("todoer dep add todo/release-v1.md todo/rust/port-the-cli-parser.md");
    s.run("todoer dep show todo/rust/port-the-cli-parser.md");
    s.run("todoer dep rm todo/release-v1.md todo/rust/port-the-cli-parser.md");
    s.run("todoer dep rm todo/release-v1.md todo/rust/port-the-cli-parser.md");
    s.run("todoer label add todo/release-v1.md release");
    s.run("todoer label add todo/release-v1.md rust-port");
    s.run("todoer label rm todo/release-v1.md release");
    s.run("todoer label add todo/release-v1.md Release");
    s.run("todoer labels");
    s.run("todoer progress");
    s.run("todoer progress rust-port");
    s.run("todoer progress missing");
    s.run("todoer mv todo/rust/port-the-config.md todo/config/");
    s.note_file("todo/release-v1.md");
    s.run("todoer mv todo/release-v1.md todo/release.txt");
    s.run("todoer check");
    s.write(
        "todo/bad.md",
        "---\ntitle: Bad\nstatus: later\nassignee: bob\nlabels: [Rust]\ndependsOn: [todo/gone.md, todo/bad.md]\n---\n",
    );
    s.run("todoer check");
    s.run("todoer ls");
    s.assert_snapshot("todos");
}

#[test]
fn claims() {
    let mut s = Session::new("claims");
    s.run("todoer add -t First -p todo");
    s.run("todoer add -t Second -p todo -d todo/first.md");
    commit(&s, "todos");
    s.run("todoer claim todo/second.md");
    s.run("todoer claim todo/first.md");
    s.run("todoer claim todo/first.md");
    s.run("git log --format=%s -1 && git status --short");
    s.note_file("todo/first.md");
    s.run("git config user.email other@example.com && todoer claim todo/first.md");
    s.run("todoer claim todo/second.md --force");
    s.run("git config user.email me@example.com && todoer ls --mine");
    s.run("todoer unclaim todo/first.md");
    s.run("todoer unclaim todo/first.md");
    s.run("echo change >> todo/first.md && todoer claim todo/first.md");
    s.quiet("git checkout todo/first.md");
    s.run("todoer add -t Third -p todo && todoer claim todo/third.md && git status --short");
    let feature = worktree(&s, "feature");
    s.run_in("todoer claim ../repo/todo/first.md", &feature);
    s.run_in(
        "todoer claim todo/first.md && git -C ../repo log --format=%s -1",
        &feature,
    );
    s.quiet("git checkout -q -b other");
    s.run("todoer unclaim todo/first.md");
    s.assert_snapshot("claims");
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
    s.quiet("todoer add -t 'Fix the sum' -p todo");
    commit(s, "start");
    s.quiet("todoer claim todo/fix-the-sum.md");
    let work = worktree(s, "fix-sum");
    s.write(
        "../fix-sum/src/app.rs",
        "fn main() {\n    let a = 1;\n    let b = 3;\n    println!(\"{}\", a + b);\n}\n",
    );
    s.write("../fix-sum/docs/guide.md", "# Guide\n\nline 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n");
    s.quiet("cd ../fix-sum && git mv src/old.rs src/lib.rs && git add -A && git commit -q -m 'Use three for b'");
    work
}

#[test]
fn pull_requests() {
    let mut s = Session::new("prs");
    s.run("todoer pr ls");
    s.run("todoer pr submit");
    let work = setup_pr(&s);
    s.write("../fix-sum/dirty.txt", "x\n");
    s.run_in("todoer pr submit", &work);
    s.quiet("rm ../fix-sum/dirty.txt");
    s.run_in(
        "todoer pr submit -m 'b is three now.' --todo todo/fix-the-sum.md",
        &work,
    );
    s.run(
        "git log --format=%s -2 && git status --short && ls -a .todoer .todoer/prs/use-three-for-b",
    );
    s.note_file(".todoer/prs/use-three-for-b/pr.md");
    s.run("todoer pr ls");
    s.run("todoer pr ls --json");
    s.run("todoer pr summary use-three-for-b");
    s.run("todoer pr details .todoer/prs/use-three-for-b/");
    s.run("todoer pr details use-three-for-b docs/guide.md --context 2");
    s.run("todoer pr details use-three-for-b src/app.rs --context 1 --lines 3-3");
    s.run("todoer pr details use-three-for-b nope.rs");
    s.run("todoer pr comment use-three-for-b src/app.rs:3 'Use a constant.'");
    s.run("todoer pr comment use-three-for-b src/app.rs:2-4 'Format this.'");
    s.run("todoer pr comment use-three-for-b src/app.rs:old:3 'Why two?'");
    s.run("todoer pr comment use-three-for-b src/lib.rs 'Name it math.rs.'");
    s.run("todoer pr comment use-three-for-b src/ 'Move under lib/.'");
    s.run("todoer pr comment use-three-for-b 'Add a test.'");
    s.run("echo 'From stdin.' | todoer pr comment use-three-for-b docs/guide.md:12");
    s.run("todoer pr comment use-three-for-b docs/guide.md:3 'Outside the changes.'");
    s.run("todoer pr comment use-three-for-b todo/fix-the-sum.md:2 'Not in the pull request.'");
    s.run("todoer pr comment use-three-for-b src/app.rs:40 'x'");
    s.run("todoer pr comment use-three-for-b src/nope.rs 'x'");
    s.run("todoer pr comment use-three-for-b ../x 'x'");
    s.run("todoer pr comment use-three-for-b src/app.rs:1");
    s.run("todoer pr summary use-three-for-b");
    s.run("todoer pr details use-three-for-b");
    s.run("todoer pr request-changes use-three-for-b -m 'Close. See the comments.'");
    s.run("git log --format=%s -1 && git status --short");
    s.note_file(".todoer/prs/use-three-for-b/review.md");
    s.write("../fix-sum/src/app.rs", "fn main() {\n    let a = 1;\n    let b = B;\n    println!(\"{}\", a + b);\n}\n\nconst B: i32 = 3;\n");
    s.quiet("cd ../fix-sum && git commit -q -am 'Add a constant'");
    s.run("echo 'A draft.' | todoer pr comment use-three-for-b src/app.rs:2");
    s.run_in("todoer pr submit --update", &work);
    s.quiet("git checkout -q -- .todoer/prs/use-three-for-b/review.md");
    s.run_in("todoer pr submit -m 'A constant.'", &work);
    s.run_in("todoer pr submit -m 'A constant.' --update", &work);
    s.run("git log --format=%s -1 && git status --short && ls .todoer/prs/use-three-for-b");
    s.note_file(".todoer/prs/use-three-for-b/pr.md");
    s.run("todoer pr summary use-three-for-b");
    s.run("todoer pr details use-three-for-b --rev 1");
    s.run_in("todoer pr submit", &work);
    s.run("todoer pr comment use-three-for-b src/app.rs:7 'Why i32?'");
    s.run("todoer pr comment use-three-for-b docs/guide.md 'Say what the guide is for.'");
    s.run("todoer pr request-changes use-three-for-b");
    s.write("../fix-sum/docs/guide.md", "# Guide\n\nWhat the guide is for.\n\nline 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\n");
    s.quiet("cd ../fix-sum && git commit -q -am 'Explain the guide'");
    s.run_in("todoer pr submit --update", &work);
    s.run("todoer pr comment");
    s.run("todoer pr approve");
    s.run("todoer pr approve use-three-for-b");
    s.run("echo staged > staged.txt && git add staged.txt && todoer pr merge use-three-for-b");
    s.quiet("git reset -q staged.txt && rm staged.txt");
    s.run("todoer pr merge use-three-for-b");
    s.run("git log --format=%s -5 && git status --short && git worktree list | wc -l && cat src/app.rs");
    s.note_file("todo/fix-the-sum.md");
    s.run("todoer pr ls");
    s.assert_snapshot("pull_requests");
}

#[test]
fn closing_and_conflicts() {
    let mut s = Session::new("close");
    let work = setup_pr(&s);
    s.run_in("todoer pr submit -t 'First try'", &work);
    s.run("todoer pr close first-try");
    s.run("git log --format=%s -1 && ls .todoer/prs");
    s.run_in("todoer pr submit -t 'Second try'", &work);
    s.write("src/app.rs", "fn main() {}\n");
    commit(&s, "change app on main");
    s.run("todoer pr merge second-try");
    s.run("git log --format=%s -1 && git status --short");
    s.quiet("git checkout -q -b elsewhere");
    s.run_in("todoer pr submit", &work);
    s.run("todoer pr approve second-try");
    s.run("todoer pr comment second-try 'x'");
    s.quiet("git checkout -q main");
    s.run("todoer pr comment second-try 'Add a test.'");
    s.run("todoer pr merge second-try");
    s.run("todoer pr request-changes second-try");
    s.run("todoer pr merge second-try");
    s.assert_snapshot("closing_and_conflicts");
}

#[test]
fn code_owners() {
    let mut s = Session::new("owners");
    s.write(
        ".todoer/teams/backend.md",
        "# Backend\n\n- me@example.com\n- agent-1@example.com\n",
    );
    s.write(".todoer/teams/writers.md", "- docs@example.com\n");
    s.write(
        ".todoer/codeowners",
        "*.rs @backend\n/docs/ @writers\nsrc/old.rs other@example.com\n",
    );
    commit(&s, "owners");
    let work = setup_pr(&s);
    s.run_in("todoer pr submit", &work);
    s.run("todoer pr summary use-three-for-b");
    s.run("todoer pr summary use-three-for-b --mine");
    s.run("todoer pr summary use-three-for-b --owner writers");
    s.run("todoer pr summary use-three-for-b --owner other@example.com");
    s.run("todoer pr summary use-three-for-b --unowned");
    s.run("todoer pr summary use-three-for-b --mine --unowned");
    s.run("todoer pr details use-three-for-b --owner @writers");
    s.run("todoer pr ls --mine");
    s.run("git config user.email nobody@example.com && todoer pr ls --mine");
    s.assert_snapshot("code_owners");
}

#[test]
fn skill_and_settings() {
    let mut s = Session::new("skill");
    s.run("todoer skill | head -5");
    s.run("todoer skill --init");
    s.run("todoer skill --init");
    s.run(
        "ls .claude/skills/todoer .agents/skills/todoer && head -3 .claude/skills/todoer/SKILL.md",
    );
    s.write("todoer.json", "{\"mainBranch\": \"trunk\", \"port\": 1}\n");
    s.run("todoer ls");
    s.write("todoer.json", "{\"mainBranch\": \"trunk\"}\n");
    s.run("todoer add -t A && todoer claim a.md");
    s.write("todoer.json", "{\"mainBranch\": \"\"}\n");
    s.run("todoer ls");
    s.assert_snapshot("skill_and_settings");
}
