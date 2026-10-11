# todoer - Commands

Run every command in a git project, in the main checkout or in any of its worktrees. todoer finds
the main checkout from `git worktree list` and keeps all its data there, in `.todoer/`.

## Todos

| Command                               | Does                                             |
| ------------------------------------- | ------------------------------------------------ |
| `todoer init`                         | writes `todoer.json` and `.todoer/`              |
| `todoer ls [--status <list>] [--all]` | lists the open todos; `--all` adds the done ones |
| `todoer add "<title>" [--body <t>]`   | adds a todo and prints its id                    |
| `todoer show <id>`                    | prints a todo, with the full path of each image  |
| `todoer status <id> <status>`         | sets `todo`, `in-progress`, `review` or `done`   |
| `todoer attach <id> <image>`          | attaches a PNG, JPEG, GIF or WebP image          |
| `todoer rm <id>`                      | removes a todo and its images                    |
| `todoer ui [--port <p>] [--open]`     | serves the page on `127.0.0.1` and `::1`         |

`ls` takes `--group <name>` (repeat it for more groups) and `--ready` (the todos that can start
now). `add` takes `--group <name>`, also more than once.

A todo id has 4 characters, like `7f3a`. Todos and pull requests never share an id. `--json` on
`ls`, `add` and `show` prints JSON.

## Groups

A group is a name for related todos, like `tests refactoring` or
`bump some-npm-package version to 0.3.5`. A todo can be in many groups. A group exists while one
todo is in it.

| Command                                | Does                                                 |
| -------------------------------------- | ---------------------------------------------------- |
| `todoer group ls`                      | lists the groups: done of total, ready, waiting, PRs |
| `todoer group show "<name>"`           | prints the todos of a group and what they wait for   |
| `todoer group add "<name>" <id> [...]` | puts todos in a group; a new name makes the group    |
| `todoer group rm "<name>" <id> [...]`  | takes todos out of a group                           |
| `todoer group rm "<name>"`             | removes the group from every todo; the todos stay    |
| `todoer group rename "<old>" "<new>"`  | renames a group in every todo                        |

## Dependencies

A todo or a pull request can depend on other todos and pull requests. There is no limit on the
depth. An item waits until each todo that it depends on is `done` and each pull request is `merged`.
A pull request also waits for the dependencies of its todos.

| Command                          | Does                                                          |
| -------------------------------- | ------------------------------------------------------------- |
| `todoer dep add <id> <id> [...]` | the first item depends on each other item                     |
| `todoer dep rm <id> <id> [...]`  | removes those dependencies                                    |
| `todoer dep show <id>`           | prints the tree of what the item waits for, and what needs it |

- An id is a todo id or a pull request id. Write `todo:<id>` or `pr:<id>` when you must be clear.
- todoer refuses a dependency that makes a cycle. A pull request cannot depend on its own todos.
- `todoer start` and `todoer pr merge` refuse an item that waits. `--force` skips the check.
- When you remove a todo, todoer removes it from the dependencies of other items.
- `dep show` prints an item that two paths reach once. The second path says `see above`.

## Work and pull requests

| Command                                       | Does                                                 |
| --------------------------------------------- | ---------------------------------------------------- |
| `todoer start <id> [<id> ...]`                | makes the branch `todoer/<ids>` and its worktree     |
| `todoer start <id> [<id> ...] --here`         | claims the todos for the worktree that you are in    |
| `todoer pr submit [--title <t>] [-m <text>]`  | sends the commits of the worktree as a pull request  |
| `todoer pr ls [--all]`                        | lists the open pull requests                         |
| `todoer pr show <id> [--diff] [--revision n]` | prints a pull request and its review comments        |
| `todoer pr comment <id> <target> "<text>"`    | adds a review comment                                |
| `todoer pr merge <id> [--keep]`               | applies the patch on the main branch                 |
| `todoer pr close <id>`                        | closes a pull request without a merge                |
| `todoer pr prompt <id>`                       | prints the prompt that asks an agent to fix a review |
| `todoer skill [--init]`                       | prints the agent workflow, or writes it as a skill   |

`todoer start` makes the worktree in `worktreesDir`. Under Claude Code (the `CLAUDECODE` environment
variable is set) the default is `.claude/worktrees`, otherwise `.todoer/worktrees`. todoer adds the
folder to `.git/info/exclude`.

`todoer pr submit` must run in a linked worktree, on a branch that is not the main branch, with no
uncommitted change. It takes the commits after the merge base with the main branch. A second submit
on the same branch adds the next revision to the same pull request.

`todoer pr merge` must run with the main branch checked out in the main checkout, and with no staged
change. It runs `git am --3way`. When the patch does not apply, todoer runs `git am --abort` and the
main branch stays as it was. Without `--keep`, the merge removes the worktree and the branch.

## Review targets

| Target              | Means                                                |
| ------------------- | ---------------------------------------------------- |
| `src/app.js:42`     | line 42 of the new version of the file               |
| `src/app.js:old:42` | line 42 of the old version, a removed line           |
| `src/app.js`        | the whole file, for example "rename it" or "move it" |
| `src/legacy/`       | the whole folder, for example "move it under `lib/`" |

## Settings - `todoer.json`

| Key            | Value                                        | Default                                    |
| -------------- | -------------------------------------------- | ------------------------------------------ |
| `maxImageKb`   | the largest image, in kB (1 kB = 1000 bytes) | `150`                                      |
| `mainBranch`   | the branch that pull requests go to          | the branch of the main checkout            |
| `worktreesDir` | the folder for new worktrees, relative       | `.todoer/worktrees` or `.claude/worktrees` |
| `agentCommand` | the command that runs an agent from the page | none                                       |
| `port`         | the port of `todoer ui`                      | `4507`                                     |

An unknown key or a bad value stops every command, and the message names the key.

`agentCommand` is an array. todoer puts the prompt in place of `{prompt}`, or adds it at the end.
The agent runs in the main checkout, and its output goes to `.todoer/runs/<time>.log`. For example:
`["claude", "--permission-mode", "acceptEdits", "-p", "{prompt}"]`. The agent must be able to run
git and todoer without a question.
