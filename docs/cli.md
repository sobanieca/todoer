# todoer - Commands

Run every command in a git project, in the main checkout or in any of its worktrees. todoer finds
the main checkout from `git worktree list`. A path argument is relative to the current folder.
todoer prints paths relative to the repository root.

## Todos

| Command                                                      | Does                                            |
| ------------------------------------------------------------ | ----------------------------------------------- |
| `todoer ls [--all] [--mine] [--label l] [--ready] [--json]`  | lists the active todos                          |
| `todoer add -t <title> [-p dir] [-l label] [-d path] [text]` | adds a todo and prints its path                 |
| `todoer done <path>...`                                      | sets `status: done`                             |
| `todoer mv <old> <new>`                                      | moves a todo and changes each `dependsOn` to it |
| `todoer check`                                               | reports bad todos and exits with `1`            |
| `todoer claim <path> [--force]`                              | claims a todo, with a commit on the main branch |
| `todoer unclaim <path>`                                      | frees a todo, with a commit on the main branch  |

- todoer finds the todos with `git ls-files -co --exclude-standard '*.md'`. A todo has a `status`
  field in its front matter. todoer skips `.todoer/`.
- `ls` shows the state of each todo: `in-progress`, `ready`, `blocked` (it waits for a todo that is
  not `done` or `dropped`), `done` or `dropped`. `--all` adds the done and the dropped todos.
- `add` writes `<slug of the title>.md` in the folder of `-p`, or in the current folder. Give `-l`
  and `-d` once for each label and each dependency. When there is no text and stdin is not a
  terminal, `add` reads the text from stdin.
- `check` reports an unknown status, a bad label, a bad assignee, a missing `dependsOn` path and a
  cycle. Every command that reads all the todos prints the same problems as warnings.
- Todo commands change the files of the current checkout. Commit them as other files.

### Claims

`claim` sets `status: in-progress` and `assignee: <git config user.email>`. `unclaim` sets
`status: todo` and removes the assignee. Both change the todo in the main checkout, and commit only
that file on the main branch.

- The main checkout must have the main branch checked out.
- `claim` refuses a todo of another person, a `done` or `dropped` todo, a todo that waits for
  another todo (`--force` skips this check), and a todo with uncommitted changes in the main
  checkout.
- A second claim waits for the first one, then reads the todo again.

## Dependencies and labels

| Command                            | Does                                                          |
| ---------------------------------- | ------------------------------------------------------------- |
| `todoer dep add <path> <path>...`  | the first todo depends on each other todo                     |
| `todoer dep rm <path> <path>...`   | removes those dependencies                                    |
| `todoer dep show <path>`           | prints the tree of what the todo waits for, and what needs it |
| `todoer label add <path> <label>`  | adds a label to a todo                                        |
| `todoer label rm <path> <label>`   | removes a label from a todo                                   |
| `todoer labels [--json]`           | lists the labels with the number of todos                     |
| `todoer progress [label] [--json]` | prints the progress of a label, or one line for each label    |

- todoer refuses a dependency that makes a cycle. There is no depth limit.
- `dep show` prints a todo that two paths reach once. The second path says `see above`.
- A label matches `^[a-z0-9-]+$`. A label exists while one todo has it.
- `progress` counts the `done` todos of all the todos with the label, but not the `dropped` todos.
  It lists the open todos in dependency order: in progress, ready and blocked. A blocking todo
  without the label shows `(outside label)`.

## Pull requests

| Command                                           | Does                                                 |
| ------------------------------------------------- | ---------------------------------------------------- |
| `todoer pr submit [-t title] [-m text] [--todo path]... [--update]` | sends the commits of the branch as the pull request |
| `todoer pr ls [--mine] [--json]`                  | lists the open pull requests                         |
| `todoer pr merge <pr>`                            | applies the patch on the main branch                 |
| `todoer pr close <pr>`                            | closes a pull request without a merge                |

A `<pr>` is the slug or the path of the folder.

- `submit` runs in a worktree, on a branch that is not the main branch, with no uncommitted
  change. It takes the commits after the merge base with the main branch. `--todo` names a todo
  of the pull request.
- A pull request keeps only the patch of its latest submit. A second submit from the same branch
  replaces the patch and the diff, and removes the review. git history keeps the old patch and the
  old review.
- When the review has comments, a second submit stops. Do what the comments ask, then run
  `todoer pr submit --update`. todoer does not check the comments.
- A second submit also stops when a reviewer has draft comments. The reviewer must send the review
  first.
- `-m` on a second submit writes the `## Update` section of `pr.md`. Tell the reviewer what you
  changed there.
- `merge` needs the main branch in the main checkout and no staged change. It stops when the
  review has a comment, also a draft. A comment is resolved when `pr submit --update` removes the
  review. It runs
  `git am --3way`. When the patch does not apply, the main branch does not change. Then todoer sets
  `status: done` on the todos of the pull request and removes the folder, in one commit.
- `close` removes the folder in one commit. The todos keep their status.
- `ls --mine` lists the pull requests that change a file that I own.

## Review

| Command                                                    | Does                                       |
| ---------------------------------------------------------- | ------------------------------------------ |
| `todoer pr summary <pr>`                                   | the files, the owners, the comment counts  |
| `todoer pr details <pr> [file...] [--context N] [--lines 40-80]` | the diff with line numbers and comments |
| `todoer pr comment <pr> [target] [text]`                   | adds a draft comment                       |
| `todoer pr approve <pr> [-m text]`                         | sets the verdict and commits the review    |
| `todoer pr request-changes <pr> [-m text]`                 | sets the verdict and commits the review    |
| `todoer pr review <pr>`                                    | the summary and all the details            |

- `summary`, `details` and `review` take `--owner <team|email>`, `--mine` or `--unowned`.
- `details` shows the old and the new line numbers. `--context N` reads the old file from the base
  and shows `N` more lines around each change. `--lines 40-80` shows only these new lines.
- Colors and `$PAGER` are on only when the output is a terminal. `NO_COLOR` turns colors off.
- The review, `review.md`, is for the latest patch only. `todoer pr submit --update` removes it,
  and the next review starts with no comments.
- A new comment is a draft: a change in `review.md` that is not committed. `approve` and
  `request-changes` commit the review file, so each review is one commit.
- With one argument after `<pr>`, `comment` uses it as a target when it names a line, a file or a
  folder of the diff. Then it reads the text from stdin. Else the argument is the text of a comment
  on the whole pull request.

| Target          | Means                                      |
| --------------- | ------------------------------------------ |
| `src/a.rs:42`   | line 42 of the new version                 |
| `src/a.rs:40-45`| lines 40 to 45 of the new version          |
| `src/a.rs:old:12` | line 12 of the old version, a removed line |
| `src/a.rs`      | the whole file                             |
| `src/cli/`      | the whole folder                           |
| none            | the whole pull request                     |

A new line can be any line of any file in the new version: a file of the pull request, or a file
of the base that the pull request does not change. Use this to point to code that must change too.
An old line must be in the diff. `pr details` shows a line outside the changes with two lines
around it, and `pr summary` lists the files that the pull request does not change, with their
comments.

### Review with an agent

Review a pull request by talking with an agent. The agent reads the patch with `pr details`, and
adds the comments that you ask for with `pr comment`. Then it sends the review when you ask.

## Code owners

- `.todoer/teams/<team>.md` lists the members of a team by git email. A member can be an agent.
- `.todoer/codeowners` uses the CODEOWNERS syntax. When it is missing, todoer reads
  `.github/CODEOWNERS`. The patterns use the gitignore rules. The last rule that matches wins.
- `@team` names a team. An email names a person.
- "Me" is `git config user.email` and each team that has this email.
- For a renamed file, the owners of the old and the new path count.

## Other commands

| Command                   | Does                                               |
| ------------------------- | -------------------------------------------------- |
| `todoer skill [--init]`   | prints the agent workflow, or writes it as a skill |
| `todoer update`           | installs the latest release                        |
| `todoer help`             | prints the help                                    |
| `todoer version`          | prints the version                                 |

`skill --init` writes `.claude/skills/todoer/SKILL.md` and `.agents/skills/todoer/SKILL.md`.

## Settings - `todoer.json`

| Key            | Value                                   | Default                                      |
| -------------- | --------------------------------------- | -------------------------------------------- |
| `mainBranch`   | the branch for the commits of todoer    | `origin/HEAD`, else `main`, else `master`    |

An unknown key or a bad value stops every command, and the message names the key.
