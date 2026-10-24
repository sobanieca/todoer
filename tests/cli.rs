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
    s.run("todoer pr show --help");
    s.run("todoer pr apply x --no-commit");
    s.run("todoer done");
    s.run("cd .. && todoer ls");
    s.assert_snapshot("help_and_errors");
}

#[test]
fn todos() {
    let mut s = Session::new("todos");
    s.run("todoer add port-cli-parser 'Parse the arguments.'");
    s.run("todoer add port-config");
    s.run("echo 'From stdin.' | todoer add release-v1");
    s.run("todoer add port-cli-parser");
    s.run("todoer add");
    s.run("todoer add Fix_Login");
    s.run("todoer add x -l label");
    s.note_file(".todoer/todos/port-cli-parser.md");
    s.note_file(".todoer/todos/release-v1.md");
    s.write(".todoer/todos/notes.txt", "Not a todo.\n");
    s.write(".todoer/todos/drafts/old.md", "Not a todo.\n");
    s.run("todoer ls");
    s.run("todoer ls --json");
    s.run("todoer ls --mine");
    s.run("todoer show port-cli-parser");
    s.run("cd .todoer/todos && todoer show port-cli-parser.md --json");
    s.run("todoer show missing");
    s.run("todoer claim port-config");
    s.run("todoer show");
    s.run("todoer edit port-config");
    s.run("todoer labels");
    s.run("todoer check");
    s.run("todoer mv port-config config-file");
    s.run("todoer done port-cli-parser");
    s.run("todoer ls");
    s.write(".todoer/todos/Bad.md", "A bad name.\n");
    s.run("todoer ls");
    s.run("git status --short && git log --format=%s");
    s.assert_snapshot("todos");
}

#[test]
fn todos_of_many_projects() {
    let mut s = Session::new("projects");
    s.quiet("todoer add fix-login");
    s.quiet("mkdir ../other && cd ../other && git init -q && git commit -q --allow-empty -m init && git checkout -q -b feat/some-feature && todoer add write-docs");
    s.quiet("mkdir ../empty && cd ../empty && git init -q");
    s.quiet("mkdir -p ../work/api && cd ../work/api && git init -q && git commit -q --allow-empty -m init && todoer add add-rate-limit");
    s.quiet("cd ../other && git worktree add -q ../other-docs");
    s.run("cd .. && todoer ls");
    s.run("cd .. && todoer ls --json");
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
    s.quiet("todoer add fix-the-sum 'Make the sum right.'");
    commit(s, "start");
    let work = worktree(s, "fix-sum");
    s.write(
        "../fix-sum/src/app.rs",
        "fn main() {\n    let a = 1;\n    let b = 3;\n    println!(\"{}\", a + b);\n}\n",
    );
    s.write("../fix-sum/docs/guide.md", "# Guide\n\nline 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n");
    s.quiet("cd ../fix-sum && git mv src/old.rs src/lib.rs && todoer done fix-the-sum && git add -A && git commit -q -m 'Use three for b'");
    work
}

const PR: &str = ".todoer/prs/use-three-for-b";

#[test]
fn pull_requests() {
    let mut s = Session::new("prs");
    s.run("todoer pr ls");
    let work = setup_pr(&s);
    s.run("todoer pr submit");
    s.run("todoer pr submit --branch nope");
    s.run("todoer pr submit --branch main");
    s.run_in("todoer pr submit --branch main -n use-three-for-b", &work);
    s.run("todoer pr submit --branch fix-sum");
    s.run("todoer pr submit --branch fix-sum -n use-three-for-b -m 'b is three now.'");
    s.run("git log --format=%s -1 && git status --short && ls -a .todoer/prs/use-three-for-b");
    s.note_file(&format!("{PR}/pr.md"));
    commit(&s, "Submit use-three-for-b");
    s.run_in("todoer pr ls", &work);
    s.run("todoer pr ls");
    s.run("todoer pr ls --json");
    s.run("todoer ls && todoer ls --json");
    s.run("todoer pr show use-three-for-b --summary");
    s.run("todoer pr show .todoer/prs/use-three-for-b/");
    s.run("todoer pr show use-three-for-b docs/guide.md --context 2");
    s.run("todoer pr show use-three-for-b src/app.rs --context 1 --lines 3-3");
    s.run("todoer pr show use-three-for-b nope.rs");
    s.run("todoer pr show use-three-for-b src/app.rs --summary");
    let comments = [
        "src/app.rs:3 'Use a constant.'",
        "src/app.rs:2-4 'Format this.'",
        "src/app.rs:old:3 'Why two?'",
        "src/lib.rs 'Name it math.rs.'",
        "src/ 'Move under lib/.'",
        "'Add a test.'",
        "docs/guide.md:3 'Outside the changes.'",
        ".todoer/todos/fix-the-sum.md:old:1 'Is it done?'",
        ".todoer/todos/fix-the-sum.md:1 'Not in the new version.'",
        "src/app.rs:40 'x'",
        "src/nope.rs 'x'",
        "../x 'x'",
        "src/app.rs:1",
    ];
    for comment in comments {
        s.run(&format!("todoer pr comment use-three-for-b {comment}"));
    }
    s.run("echo 'From stdin.' | todoer pr comment use-three-for-b docs/guide.md:12");
    s.run("todoer pr show use-three-for-b --summary");
    s.run("todoer pr show use-three-for-b");
    s.run("git status --short");
    s.run("todoer show fix-the-sum");
    s.note_file(&format!("{PR}/review.md"));
    s.quiet(&format!(
        "git add {PR}/review.md && git commit -q -m 'Review use-three-for-b' -- {PR}/review.md"
    ));
    s.run("todoer pr ls");
    s.write("../fix-sum/src/app.rs", "fn main() {\n    let a = 1;\n    let b = B;\n    println!(\"{}\", a + b);\n}\n\nconst B: i32 = 3;\n");
    s.quiet("cd ../fix-sum && git commit -q -am 'Add a constant'");
    s.run("todoer pr submit --branch fix-sum -n use-three-for-b -m 'A constant.'");
    s.run("todoer pr submit --branch fix-sum -n use-three-for-b --update -m 'A constant.'");
    s.run("git status --short && ls .todoer/prs/use-three-for-b");
    s.note_file(&format!("{PR}/pr.md"));
    commit(&s, "Update use-three-for-b");
    s.run("todoer pr show use-three-for-b --summary");
    s.run("todoer pr comment use-three-for-b src/app.rs:7 'Why i32?'");
    s.run("todoer pr comment use-three-for-b docs/guide.md 'Say what the guide is for.'");
    s.run("todoer pr apply use-three-for-b --merge");
    commit(&s, "Review again");
    s.write("../fix-sum/docs/guide.md", "# Guide\n\nWhat the guide is for.\n\nline 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\nline 11\n");
    s.quiet("cd ../fix-sum && git commit -q -am 'Explain the guide'");
    s.run("todoer pr submit --branch fix-sum --update");
    s.run("todoer pr submit --branch fix-sum -n use-three-for-b --update");
    commit(&s, "Update use-three-for-b");
    s.run("todoer pr comment");
    s.run("todoer pr apply use-three-for-b --merge");
    s.run("git status --short && ls .todoer && cat src/app.rs");
    s.quiet("git commit -q -m 'todoer: merge use-three-for-b'");
    s.run("git log --format=%s -3 && git show --stat --format='%an <%ae>%n%B' HEAD");
    s.run("todoer pr ls && todoer ls");
    s.assert_snapshot("pull_requests");
}

#[test]
fn merging_and_closing() {
    let mut s = Session::new("merge");
    s.write("notes.txt", "notes\n");
    commit(&s, "notes");
    let work = setup_pr(&s);
    s.run("todoer pr submit --branch fix-sum -n 'First try'");
    s.run("todoer pr submit --branch fix-sum -n first-try");
    commit(&s, "Submit first-try");
    s.run("todoer pr close first-try");
    s.run("git status --short && ls .todoer");
    commit(&s, "Close first-try");
    s.quiet("echo more >> ../fix-sum/notes.txt");
    s.run("todoer pr submit --branch fix-sum -n second-try");
    s.quiet("cd ../fix-sum && git checkout -q -- notes.txt");
    s.run("todoer pr submit --branch fix-sum -n second-try");
    commit(&s, "Submit second-try");
    s.quiet("cd ../fix-sum && git switch -q -c fix-sum-2");
    s.run("todoer pr submit --branch fix-sum-2 -n second-try");
    s.run("todoer pr submit --branch fix-sum-2 -n nope --update");
    s.run("todoer pr submit --branch fix-sum-2 -n second-try --update -m 'From a new branch.'");
    commit(&s, "Update second-try");
    s.run_in("todoer pr ls", &work);
    s.run("todoer pr ls && git log --format=%s -1 && git status --short");
    s.quiet("git tag picked");
    s.quiet("echo more >> notes.txt");
    s.run("todoer pr apply second-try --merge");
    s.run("git log --format=%s -1 && git status --short && cat src/app.rs");
    s.quiet("git reset -q --hard picked && git clean -qfd src");
    s.write("src/app.rs", "fn main() {}\n");
    commit(&s, "change app on main");
    s.run("todoer pr apply second-try --merge");
    s.run("git log --format=%s -1 && git status --short");
    s.quiet("git reset -q --hard picked");
    s.run("todoer pr comment second-try 'Add a test.'");
    s.run("todoer pr apply second-try --merge");
    s.run("todoer pr apply second-try --merge --no-commit");
    s.run("todoer pr close second-try && git status --short && todoer pr ls");
    s.assert_snapshot("merging_and_closing");
}

#[test]
fn submitting_into_a_feature_branch() {
    let mut s = Session::new("into");
    let work = setup_pr(&s);
    s.quiet("git switch -q -c feature");
    s.run("todoer pr submit --branch fix-sum -n use-three-for-b");
    commit(&s, "Submit use-three-for-b");
    s.run("todoer pr ls && git switch -q main && todoer pr ls");
    s.quiet("echo more >> ../fix-sum/src/app.rs");
    s.run("git switch -q feature && todoer pr submit --branch fix-sum -n other");
    s.quiet("cd ../fix-sum && git checkout -q -- src/app.rs");
    s.run_in("todoer pr submit --branch feature -n other", &work);
    s.run("todoer pr apply use-three-for-b --merge");
    s.run("git status --short && todoer ls");
    s.quiet("git commit -q -m 'todoer: merge use-three-for-b'");
    s.quiet("todoer add follow-up && git add -A && git commit -q -m 'Plan follow-up'");
    s.quiet("git switch -q -c later && echo later > later.txt && git add -A && git commit -q -m later && git switch -q feature");
    s.run("todoer pr submit --branch later -n later");
    commit(&s, "Submit later");
    s.run("git switch -q main && todoer ls && todoer pr submit --branch feature -n feature");
    s.run("grep -E '^diff' .todoer/prs/feature/pr.patch");
    s.run("todoer ls && todoer pr show feature --summary");
    commit(&s, "Submit feature");
    s.run("todoer pr apply feature --merge");
    s.run("git status --short && todoer ls && todoer pr ls");
    s.assert_snapshot("submitting_into_a_feature_branch");
}

#[test]
fn applying_a_pull_request() {
    let mut s = Session::new("apply");
    setup_pr(&s);
    s.run("todoer pr submit --branch fix-sum -n use-three-for-b");
    commit(&s, "Submit use-three-for-b");
    s.quiet("git switch -q -c tweak");
    s.run("todoer pr apply nope");
    s.quiet("echo more >> docs/guide.md");
    s.run("todoer pr apply use-three-for-b");
    s.quiet("git checkout -q -- docs/guide.md");
    s.run("todoer pr apply use-three-for-b");
    s.run("git status --short && cat src/app.rs");
    s.run("todoer pr apply use-three-for-b");
    s.quiet("git commit -q -m 'Use three for b'");
    s.write(
        "src/app.rs",
        "fn main() {\n    let a = 1;\n    let b = 4;\n    println!(\"{}\", a + b);\n}\n",
    );
    s.quiet("git commit -q -am 'Use four for b' && git switch -q main");
    s.run("todoer pr submit --branch tweak --update -m 'Four.'");
    s.run("todoer pr submit --branch tweak -n use-three-for-b --update -m 'Four.'");
    s.run("git status --short && cat .todoer/prs/use-three-for-b/pr.md");
    s.run("grep -E '^(Subject|[-+] )' .todoer/prs/use-three-for-b/pr.patch");
    commit(&s, "Update use-three-for-b");
    s.write(
        "src/app.rs",
        "fn main() {\n    let a = 1;\n    let b = 5;\n    println!(\"{}\", a + b);\n}\n",
    );
    commit(&s, "Use five for b");
    s.run("todoer pr apply use-three-for-b --merge");
    s.quiet("git clone -q --depth 1 file://$PWD ../shallow");
    let shallow = s.path("shallow");
    s.run_in("todoer pr apply use-three-for-b --merge", &shallow);
    s.run_in(
        "git switch -q -c tweak && todoer pr apply use-three-for-b",
        &shallow,
    );
    s.quiet("git switch -q -c tweak-2");
    s.run("todoer pr apply use-three-for-b");
    s.run("git status --short && cat src/app.rs");
    s.write(
        "src/app.rs",
        "fn main() {\n    let a = 1;\n    let b = 4;\n    println!(\"{}\", a + b);\n}\n",
    );
    s.quiet("git add -A && git commit -q -m 'Resolve' && git switch -q main");
    s.run("todoer pr submit --branch tweak-2 -n use-three-for-b --update");
    s.run("git rev-parse HEAD && grep base .todoer/prs/use-three-for-b/pr.md");
    s.run("grep -E '^[-+] ' .todoer/prs/use-three-for-b/pr.patch");
    s.assert_snapshot("applying_a_pull_request");
}

#[test]
fn code_areas() {
    let mut s = Session::new("areas");
    s.write(
        ".todoer/areas",
        "# the last area that matches wins\nrust: *.rs\nlegacy: src/old.rs\napp: /src/app.rs /src/main.rs\nguides: guide.md\n",
    );
    commit(&s, "areas");
    setup_pr(&s);
    s.run("todoer pr submit --branch fix-sum -n use-three-for-b");
    let runs = [
        "todoer pr show use-three-for-b --summary",
        "todoer pr show use-three-for-b --summary -a legacy,app",
        "todoer pr show use-three-for-b --area app",
        "todoer pr show use-three-for-b -a legacy -a rust",
        "todoer pr show use-three-for-b --summary -x app",
        "todoer pr show use-three-for-b -x rust,legacy,app",
        "todoer pr show use-three-for-b -a app -x app",
        "todoer pr show use-three-for-b --summary -a nope",
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
    s.run("mkdir -p src/app && cd src/app && todoer skill --init");
    s.run("todoer skill --init");
    s.run(
        "ls .claude/skills/todoer .agents/skills/todoer && head -3 .claude/skills/todoer/SKILL.md",
    );
    let notes = s.path("notes");
    std::fs::create_dir_all(&notes).unwrap();
    s.run_in("todoer skill --init", &notes);
    s.run_in("todoer skill --init", &notes);
    s.run("ls ~/.claude/skills/todoer ~/.agents/skills/todoer");
    s.assert_snapshot("skill");
}
