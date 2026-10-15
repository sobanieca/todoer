# CLAUDE.md

## What this repo is

`todoer` keeps the todos and the pull requests of a git project as plain files in `.todoer/` of the
main checkout. AI agents work in git worktrees and send their commits as patch files. A person
reviews the patches in a local web page and merges them on the main branch. todoer is one binary,
built with `deno compile` from plain JavaScript. The page is an imp project in `ui/`.

## Migration from Deno to Rust is planned

todoer moves to Rust, with the same layout as `jsonr` and `sqlr`. The new version has no page. See
`todo/migrate-to-rust.md` and the other files in `todo/`. Do not add new features to the Deno code
or to `ui/`. Fix only bugs that block work.

## todoer is unreleased - do not preserve backward compatibility

Until the version in `deno.json` reaches `1.0.0`, treat the project as never released. Rename or
remove anything, change the data layout, the API or the CLI, when a better design shows up. Never
add a compatibility shim, a fallback path or a migration note. Fix every call site, the docs and the
ADRs in the same change.

## No dependencies

**Import nothing from outside the repo**, also not in tests. Use the Deno runtime and the web
platform: `Deno.serve`, `Deno.Command`, `Deno.watchFs`, `crypto`. Run git with `Deno.Command`. A
helper that a library would give is a small module in `src/` with a test.

`ui/` is an imp project. It imports only `imp` and its subpaths. Follow `imp skill` there, and use
`imp fmt`, `imp lint` and `imp test`, not the Deno tools. `deno fmt` and `deno lint` exclude `ui/`.

## Architecture

| Module                                 | Job                                                                        |
| -------------------------------------- | -------------------------------------------------------------------------- |
| `main.js`                              | parses the arguments and runs a command                                    |
| `src/commands/`                        | one file per command group: todos, group, dep, start, pr, ui, skill, help  |
| `src/project.js`                       | finds the main checkout from any worktree, `.todoer/`, `.git/info/exclude` |
| `src/config.js`                        | reads and checks `todoer.json`                                             |
| `src/todos.js`, `src/images.js`        | the todo files, the images and the image size check                        |
| `src/work.js`                          | `todoer start`: the worktree, the branch and the claim                     |
| `src/dependencies.js`, `src/groups.js` | the dependency graph, the cycle check, waiting items, and the groups       |
| `src/prs.js`                           | submit, revisions, comments, verdicts, merge and close                     |
| `src/reviews.js`, `src/diff.js`        | the review file format, the comment targets, the diff parser               |
| `src/agent.js`, `src/skill.js`         | the prompts for agents, `agentCommand`, the skill text                     |
| `src/ui-server.js`, `src/ui-api.js`    | the local page: listen, host and origin checks, the JSON API               |
| `src/events.js`, `src/static-files.js` | change events from `.todoer/` to the page, and the built page              |
| `ui/`                                  | the page: todos, the todo editor, the pull requests and the review         |

Rules that keep the data correct:

- **The CLI and the page call the same modules.** The page never writes a file itself.
- **The main checkout owns `.todoer/`.** A command in a worktree reads and writes the `.todoer/` of
  the main checkout. Never read `.todoer/` of a worktree.
- **A pull request is self-contained.** `v<N>.patch` is the `git format-patch` series for `git am`.
  `v<N>.diff` is the combined diff for the review. The review needs no branch.
- **Patches never contain `.todoer/`.** Every git command of a submit excludes it.
- **A dependency never makes a cycle.** Every new dependency goes through `addDependency`.
- **A comment belongs to one revision.** Only the latest revision of an open pull request takes new
  comments.

## Commands

| Task                | What it does                                                         |
| ------------------- | -------------------------------------------------------------------- |
| `deno task start`   | builds the page and serves a demo project                            |
| `deno task test`    | runs the tests                                                       |
| `deno task check`   | runs `deno fmt`, `deno lint`, the tests, then imp fmt, lint and test |
| `deno task compile` | builds the `todoer` binary with `ui/dist` inside                     |

After a batch of changes, run `deno task check` and fix every issue. When you change the page, run
`deno task start` and look at each screen that you changed in a browser.

When you change a command, an option or a setting, update `src/commands/help.js`, `docs/cli.md`, the
skill text in `src/skill.js` and the README in the same change.

## ADRs

`adr/` holds the accepted decisions. Presence means accepted. When a decision changes, replace the
ADR. When it is dropped, delete it. Planned work goes in `todo/`, one file per item with a slug
name. Delete the file when the item is done.

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
- **File names use hyphens**, never underscores.
- **No comments in code.** Use clear names. Put reasons in the docs and the ADRs.
- **Functions, not classes.** Error classes are the exception.
- **User errors** are `throw new Error('lower-case message - what to do')`. `main.js` prints them
  and exits with `1`.
- **Format**: no semicolons, single quotes, 100 columns. `deno fmt` and `imp fmt` enforce it.

## Git

Commit on the branch that is checked out. Commit only when asked. `deno task check` must pass before
a commit.
