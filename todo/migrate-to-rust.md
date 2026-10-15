---
title: Migrate from Deno to Rust, and a todo is a markdown file
status: done
labels: [rust-migration, cli-reshape]
---

Replace the Deno version with a Rust version. Use the same layout as `jsonr` and `sqlr`.

todoer is unreleased. Do not port the code one to one. Port only the parts that the new design
keeps. The new design is in this file and in the other files of `todo/`.

## Layout

- `Cargo.toml` with `edition = "2024"`, `publish = false`, and a release profile with `lto`,
  `codegen-units = 1` and `strip`.
- An `xtask` crate and cargo aliases in `.cargo/config.toml`: `cargo todoer`, `cargo verify` (fmt
  check, clippy with `-D warnings`, all tests) and `cargo update-snapshots`.
- Unit tests next to the code. `tests/cli.rs` runs the binary in a temporary git repository and
  compares the output with `tests/snapshots/*.snap`.
- `src/help.txt` for the help. `src/templates/SKILL.md` for the agent skill.
- `AGENTS.md` replaces `CLAUDE.md`. Keep the rules on prose style, ASCII only and no comments.
  Rust module names use underscores, so the hyphen rule applies only to other files.
- A publish workflow that builds Linux (musl) and macOS binaries on each push to `main`.
  `install.sh` and `todoer update` download the release assets.

## Dependencies

Keep the crate list minimal, as in `jsonr` and `sqlr`: `libc` and `ureq`. Write the argument parser,
the colors, the JSON parser, the front matter parser and the diff parser in the crate. Use `libc` for
the raw terminal mode, as `sqlr` does in `src/prompt.rs`. Run git as a child process.

## A todo is a markdown file and its path is the id

A todo is a markdown file in the repository. Its front matter has a `status` field. Its id is its
path relative to the repository root.

```markdown
---
title: Port the CLI parser
status: in-progress
assignee: sobanieca@gmail.com
labels: [migration-to-rust-followup, cli]
dependsOn: [todo/rust/port-config.md]
---

The description.
```

- `status` is `todo`, `in-progress`, `done` or `dropped`.
- `assignee` is a git email. "Me" is `git config user.email`.
- Remove the `id`, `created`, `updated`, `branch`, `worktree` and `pr` fields. git has the dates.
- Find the todos with `git ls-files -co --exclude-standard '*.md'`. Keep the files with a `status`
  field in the front matter. Skip `.todoer/`.
- A command accepts a path relative to the current folder. todoer stores paths relative to the
  repository root.
- Todo commands read and change the files of the current checkout, as for any other file. `claim`
  and `unclaim` are the exceptions, see below.
- Images are plain markdown links to files near the todo. Remove `attach` and `maxImageKb`.
- A todo is on the code branch. `todoer pr merge` sets `status: done` on the todos of the pull
  request.

## Claim a todo on the main branch

Before an agent starts a todo, it claims the todo. The claim is a commit on the main branch, so
every agent and person sees it at once.

| Command                   | Does                                                            |
| ------------------------- | --------------------------------------------------------------- |
| `todoer claim <path>`     | sets `status: in-progress` and `assignee: <me>`, then commits   |
| `todoer unclaim <path>`   | sets `status: todo` and removes `assignee`, then commits        |

- Both commands run from any worktree. They change the todo in the main checkout and commit only
  that file: `git commit -- <path>`. Other staged changes stay as they are.
- The main checkout must have the main branch checked out. If it is on a feature branch, todoer
  stops and tells the person to check out the main branch there.
- todoer takes a lock file in the git common folder for each commit on the main branch. A second
  claim waits for the first, then reads the todo again.
- `claim` refuses a todo that another person claimed, a todo that is `done` or `dropped`, and a
  todo that waits for another todo. `--force` skips the wait check only.
- A claim of a todo that I already claimed does nothing.
- `claim` refuses a todo that has uncommitted changes in the main checkout. A new todo that git
  does not track yet is added in the claim commit.
- The skill tells agents to claim first, then make the worktree from the main branch.
- A remote is out of scope. See `todo/sync-with-remote.md`.

## The main branch

`mainBranch` in `todoer.json`, else the target of `origin/HEAD`, else `main`, else `master`. Every
commit that todoer makes goes on this branch, in the main checkout. todoer never commits on a
feature branch.

## .todoer/ is in git

Commit `.todoer/` like other files. Remove the code that adds it to `.git/info/exclude`.
`.todoer/.gitignore` ignores `worktrees/`. The main checkout owns `.todoer/`: a command in a worktree
reads and writes the `.todoer/` of the main checkout.

## Commands

| Command                                                      | Does                                            |
| ------------------------------------------------------------ | ----------------------------------------------- |
| `todoer ls [--all] [--mine] [--label l] [--ready] [--json]`  | lists the active todos                          |
| `todoer add -t <title> [-p dir] [-l label] [-d path] [text]` | adds a todo and prints its path                 |
| `todoer done <path>`                                         | sets `status: done`                             |
| `todoer mv <old> <new>`                                      | moves a todo and changes each `dependsOn` to it |
| `todoer check`                                               | reports bad todos, see below                    |

- `add` reads the text from stdin when stdin is not a terminal. The file name is a slug of the
  title. `-p` sets the folder, the default is the current folder.
- `--mine` compares `assignee` with `git config user.email`.
- `check` reports an unknown status, a bad label, a bad assignee, a missing `dependsOn` path and a
  cycle. It exits with `1` on an error, so CI can run it. Every command prints the same warnings
  when it reads a bad todo.

## Port

- Keep: finding the main checkout, `todoer.json`, front matter, the diff parser, the review file
  format, the patch submit and the merge, `todoer skill`.
- Do not port: the page (`ui/`, `ui-server`, `ui-api`, `events`, `static-files`), `groups`, the id
  based dependencies, `images`, `attach`, `start`, `pr prompt`, `init`.

## Remove

- `deno.json`, `main.js`, `src/*.js`, `test/`, `ui/`, `dev/`, `analysis.md`.
- The ADRs `dependency-free-deno-binary`, `local-page` and `lazy-dependency-tree`. Add an ADR for
  the Rust binary and its dependency rule.
- Replace `plain-files-in-the-main-checkout`. Delete `groups-are-names-on-todos` and
  `image-size-limit`.
