# AGENTS.md

## What this repo is

`todoer` keeps the todos and the pull requests of a git project as plain files in git.
AI agents and people work on the todos in git worktrees, and review on the command line. todoer is
one Rust binary.

- A todo is a file `.todoer/todos/<name>.md`. The name is the id and the title. The file is plain
  markdown. Todos have no assignee, no state, no labels and no dependencies.
- A todo file exists only while the work is open. `todoer done` and `todoer pr apply --merge`
  remove it. git history keeps it.
- A pull request is a folder `.todoer/prs/<name>/`, with the patch and the review of the
  latest submit. In the checkout of the branch that it goes into, `todoer pr submit --branch <work>`
  writes it in the working tree.
- todoer has no main branch and no settings file. It works on the current checkout, on any branch.
- todoer never commits. The person commits the changes.

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
| `src/commands/`                          | one file per command group: todos, pr, skill                         |
| `src/args.rs`                            | the argument parser; each command declares its flags                 |
| `src/project.rs`                         | the current checkout, the lock, the staging of a merge               |
| `src/todos.rs`, `src/frontmatter.rs`     | the todo files, and the front matter of `pr.md`                      |
| `src/prs.rs`                             | submit, merge, close, comments                                       |
| `src/review.rs`, `src/diff.rs`           | the review file format, the comment targets, the diff parser         |
| `src/pr_view.rs`                         | the text of `pr show`                                                |
| `src/areas.rs`                           | the code areas and the gitignore pattern matcher                     |
| `src/terminal.rs`                        | stdin and `$PAGER`                                                   |
| `src/git.rs`                             | git as a child process                                               |
| `src/json.rs`                            | the JSON writer, for `--json`                                        |
| `src/colors.rs`, `src/logger.rs`         | the colors, and the messages on stderr                               |
| `src/update.rs`                          | `todoer update`, and the daily check for a new version               |
| `src/help.txt`, `src/templates/SKILL.md` | the help and the agent skill                                         |

Rules that keep the data correct:

- **todoer knows no main branch.** Every command reads and writes `.todoer/` of the current
  checkout, on any branch. Never add a setting or a guess for a main branch.
- **todoer never commits.** No command runs `git commit`, `git am` or `git update-ref`. The
  squash of a submit is a `git commit-tree` object on no branch. There is no `--no-commit` option.
  A command changes the working tree of the current checkout, and prints the git command that
  commits the change when that helps. Only `pr apply` changes the index: `git apply --3way`, and
  `Project::stage` for a merge and for the removed todos.
- **A submit runs on the branch that it goes into.** `pr submit --branch <work>` runs in the
  checkout of that branch. `--branch` names the branch of the work. The submit writes the folder
  in the current working tree. No command changes a branch that is not checked out.
- **todoer does few git operations.** It reads the diff, the patch and old files, and `pr apply`
  applies a patch. It never pushes or fetches.
- **One lock.** Take `Project::lock` before a read that decides a change, and hold it until the
  write is done.
- **A pull request is self-contained.** `pr.patch` is one squashed commit, for `git am`. The
  review reads its diff, and needs no branch. `pr.md` names no
  branch: the branch that holds the folder is the branch that it goes into, and `-n <pr>` finds
  it for an update.
- **A patch carries all the work, also in `.todoer/`.** Every git command of a submit excludes
  only the folder of the pull request itself, `.todoer/prs/<name>/`.
- **Todo commands do not commit.** Every todo command changes `.todoer/todos/` of the current
  checkout, on any branch. The person commits the changes. A feature branch can hold todos that
  are not on another branch. A todoer pull request carries the todos that the work adds, changes
  and removes.
- **A done todo is gone.** `done` removes the file. The work runs `done` and commits it. The todos
  of a pull request are the todo files that its diff removes. `pr apply` removes them by name, so
  an edit of a todo on the current branch does not stop the patch.
- **There is no verdict.** A merge accepts a pull request. A committed review with comments asks
  for changes.
- **A merge needs a review with no comments.** `pr apply --merge` stops on any comment. Only
  `pr submit --update` removes the comments.
- **A failed merge changes nothing.** `pr apply --merge` tries the patch on a copy of the index
  before it touches the working tree.

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

Commit on the branch that is checked out. Commit the changes when they are complete. Do not wait
for the person to ask. Do not push. Push only when the person asks.
