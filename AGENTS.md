# AGENTS.md

## What this repo is

`todoer` keeps the todos and the pull requests of a git project as plain files in git.
AI agents and people claim todos, work in git worktrees, and review on the command line. todoer is
one Rust binary.

- A todo is a file `.todoer/todos/<name>.md`. The name is the id and the title. The front matter
  has `assignee`, `labels`, `dependsOn` and `dependsOnLabels`. All are optional.
- A todo with an assignee is in progress. A todo without an assignee is new.
- A todo file exists only while the work is open. `todoer done` and `todoer pr merge` remove it.
  git history keeps it.
- A pull request is a folder `.todoer/prs/<name>/`, with the patch, the diff and the review of the
  latest submit. `todoer pr submit --branch <b>` commits it on `b`, the branch that it goes into.
- todoer has no main branch and no settings file. It works on the current checkout, on any branch.

## todoer is unreleased - do not preserve backward compatibility

Until the version in `Cargo.toml` reaches `1.0.0`, treat the project as never released. Rename or
remove anything, change the data layout or the CLI, when a better design shows up. Never add a
compatibility shim, a fallback path or a migration note. Fix every call site, the docs and the ADRs
in the same change.

## Dependencies

Keep the crate list minimal: `ureq` (`todoer update`) only. The argument parser, the colors, the
JSON writer, the front matter parser, the diff parser and the gitignore pattern matcher are in the
crate. Run git as a child process.

## Architecture

| Module                                   | Job                                                                  |
| ---------------------------------------- | -------------------------------------------------------------------- |
| `src/main.rs`                            | the command table, `help`, `version`                                 |
| `src/commands/`                          | one file per command group: todos, deps, labels, pr, skill           |
| `src/args.rs`                            | the argument parser; each command declares its flags                 |
| `src/project.rs`                         | the current checkout, the lock, the commits of todoer                |
| `src/todos.rs`, `src/frontmatter.rs`     | the todo files, the dependency graph, the front matter               |
| `src/prs.rs`                             | submit, merge, close, comments                                       |
| `src/review.rs`, `src/diff.rs`           | the review file format, the comment targets, the diff parser         |
| `src/pr_view.rs`                         | the text of `pr summary` and `pr details`                            |
| `src/areas.rs`                           | the code areas and the gitignore pattern matcher                     |
| `src/rules.rs`                           | the label rules in `.todoer/labels/`                                 |
| `src/terminal.rs`                        | stdin and `$PAGER`                                                   |
| `src/git.rs`                             | git as a child process                                               |
| `src/json.rs`                            | the JSON writer, for `--json`                                        |
| `src/colors.rs`, `src/logger.rs`         | the colors, and the messages on stderr                               |
| `src/update.rs`                          | `todoer update`, and the daily check for a new version               |
| `src/help.txt`, `src/templates/SKILL.md` | the help and the agent skill                                         |

Rules that keep the data correct:

- **todoer knows no main branch.** Every command reads and writes `.todoer/` of the current
  checkout, on any branch. Never add a setting or a guess for a main branch.
- **todoer commits only its own files.** `claim`, `unclaim` and the pull request commands commit.
  Call `Project::plan_commit` before the change: it skips the commit for `--no-commit` and when the
  checkout has other tracked changes. Then `Project::commit` commits only the named paths.
  `pr merge` uses `Project::amend` to add its paths to the commit of the patch.
- **A submit lands on the branch that it goes into.** When that branch is checked out, `pr submit`
  writes and commits in that checkout. Else `Project::commit_to_branch` commits the folder on the
  branch with a temporary index. No other command changes a branch that is not checked out.
- **todoer does few git operations.** It reads the diff, the patch and old files, `pr merge`
  applies a patch, and `pr submit` commits its folder. It never pushes or fetches.
- **One lock.** Take `Project::lock` before a read that decides a change, and hold it until the
  write or the commit is done.
- **A pull request is self-contained.** `pr.patch` is one squashed commit, for `git am`.
  `pr.diff` is the combined diff for the review. The review needs no branch.
- **Patches never contain `.todoer/`.** Every git command of a submit excludes it.
- **A dependency never makes a cycle.** `dep add` checks `Todos::reaches` first. A label
  dependency is an edge to each other todo of the label, so `add`, `label add` and `labels mv`
  call `Todos::check_no_cycle` before they write.
- **Most todo commands do not commit.** Every todo command changes `.todoer/todos/` of the current
  checkout, on any branch. Only `claim` and `unclaim` commit, so that a claim can be pushed at
  once. The person commits the other changes. A feature branch can hold todos that are not on
  another branch. A todoer pull request cannot add or change a todo.
- **A done todo is gone.** `done` and `pr merge` remove the file, and remove its name from each
  `dependsOn`. Thus a missing name in `dependsOn` is always a problem. When no other todo has a
  label of `dependsOnLabels`, `done`, `pr merge`, `label rm` and `labels rm` remove the label from
  it. Thus a label in `dependsOnLabels` that no other todo has is always a problem.
- **There is no verdict.** A merge accepts a pull request. A committed review with comments asks
  for changes.
- **A merge needs a review with no comments.** `pr merge` stops on any comment. Only
  `pr submit --update` removes the comments.
- **A failed merge changes nothing.** `pr merge` aborts `git am` on a failure. `--no-commit` tries
  the patch on a copy of the index before it touches the working tree.

## Commands

```bash
cargo todoer [args]        # runs todoer from source
cargo verify               # fmt --check, clippy -D warnings and all tests
cargo test                 # runs all tests
cargo update-snapshots     # accepts new CLI output in tests/snapshots
```

`cargo verify` must pass before a commit. CI runs it.

When you change a command or an option, update `src/help.txt`, `docs/cli.md`, `docs/guide.html`,
the skill in `src/templates/SKILL.md` and the README in the same change.

## Testing

- Unit tests are next to the code (`#[cfg(test)]`).
- `tests/cli.rs` runs the binary in a temporary git repository and compares the output with
  `tests/snapshots/*.snap`. The commit dates are fixed, so the commit hashes do not change.

## ADRs

`adr/` holds the accepted decisions. Presence means accepted. When a decision changes, replace the
ADR. When it is dropped, delete it.

## Planned work

Planned work is todoer todos in `.todoer/todos/`, one todo per item. Add a todo with
`todoer add`. When the item is done, run `todoer done`.

## Prose style - Simplified Technical English

Write all prose in Simplified Technical English (ASD-STE100): docs, ADRs, this file, commit
messages, the skill text and user-facing strings. Keep code names, options and quoted output as they
are.

- An instruction has 20 words or fewer. A descriptive sentence has 25 words or fewer.
- Active voice, present tense, the imperative for instructions. "Must" for a requirement, "can" for
  a possibility.
- One term for one thing. Simple words. No idioms, no metaphors, no humour.

## Code style

- **ASCII only**, in every file.
- **File names use hyphens**, except Rust modules, which use underscores.
- **No comments in code.** Use clear names. Module doc comments and `SAFETY` notes are the
  exceptions. Put reasons in the docs and the ADRs.
- **User errors** are `Err(String)` with a lower-case message: "what is wrong - what to do".
  `main` prints them and exits with `1`.
- `cargo fmt` sets the format.

## Landing page

The landing page is `docs/index.html`. The guide, `docs/guide.html`, shows the workflow step by
step. When you add a feature or change a behavior, update both pages in the same change, so that
they always show the current feature set.

## Publishing

Every push to `main` builds binaries for Linux (musl, static) and macOS (x64, arm64) and publishes
them as the GitHub release `v<version>` (the version in `Cargo.toml`). `install.sh` and `todoer
update` download these assets, so their names (`todoer-linux-x64`, ...) must not change. Every push
to `main` also publishes `docs/`, `install.sh` and `src/help.txt` to GitHub Pages at
https://sobanieca.github.io/todoer/.

## Git

Commit on the branch that is checked out. Commit only when asked.
