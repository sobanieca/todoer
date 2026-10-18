# todoer - Commands

Run every command in a git project, in any checkout and on any branch. todoer reads and writes the
`.todoer/` folder of the current checkout. It does not know a main branch, and it has no settings
file.

For the workflow from a new todo to a merge, read the [guide](https://sobanieca.github.io/todoer/guide.html).

## Todos

| Command                                         | Does                                                      |
| ----------------------------------------------- | --------------------------------------------------------- |
| `todoer ls [--mine] [--label l] [--available] [--json]` | lists the todos                                       |
| `todoer add <name> [-l label] [-d name] [-D label] [text]` | adds the todo `.todoer/todos/<name>.md`        |
| `todoer done <name>...`                         | removes the todos, and removes them from each `dependsOn` |
| `todoer mv <old> <new>`                         | renames a todo and changes each `dependsOn` to it         |
| `todoer check`                                  | reports bad todos and bad label rules, and exits with `1` |
| `todoer claim <name>... [--label l] [--force] [--no-commit]` | sets the assignee to me and commits the todos |
| `todoer unclaim <name>... [--no-commit]`        | removes the assignee and commits the todos                |

- A todo command changes the files of the current checkout, on any branch. Only `claim` and
  `unclaim` commit. The other todo commands do not commit. You decide when to commit those changes.
- A command that takes a name also takes the path of the todo file, for example
  `.todoer/todos/fix-login.md`.
- `ls` shows the state of each todo: `in-progress` (it has an assignee), `available` or `blocked` (it
  waits for a todo that still exists, or for a label that another todo still has).
- `ls` also shows each open pull request that names the todo in `todos`: `pr: <name>`, with the
  number of comments. `--json` adds the field `prs`.
- `add` and `mv` refuse a bad name and print a correct name. Give `-l`, `-d` and `-D` once for each
  label and each dependency. `-D label` (`--depends-on-label`) makes the todo wait for each todo of
  the label. When there is no text and stdin is not a terminal, `add` reads the text from
  stdin.
- When a label rule exists, `add` prints `check the label rules: todoer labels` after the path.
- There is no `dropped` status. To drop a todo, run `todoer done`. git history keeps the file.
- `check` reports a bad name, a bad label, a bad assignee, a missing `dependsOn` name, a label in
  `dependsOnLabels` that no other todo has, and a cycle.
  Every command that reads all the todos prints the same problems as warnings. `check` also
  reports a label rule file with a bad name, and an empty rule.
- `claim` sets `assignee` to your git email in each todo, and commits the todo files on the current
  branch. Push the commit, so that other people see the claim. todoer never pushes.
- `claim` takes many names. `--label l` adds each todo of the label that has no assignee. A todo
  of the label that another person has stays as it is.
- `claim` refuses a todo of another person and a todo that waits for another todo. A todo that
  waits only for todos of the same claim is not refused. `--force` skips the dependency check.
- `unclaim` removes the assignee and commits the todo files.
- With `--no-commit`, the changes stay in the working tree. When the checkout has other changes
  that are not committed, `claim` and `unclaim` do not commit. They write the files and print a
  warning. Then commit the todo files yourself.

### Many projects

Outside a git repository, `todoer ls` lists the todos of each git project with `.todoer/todos/` in
the current folder and in its subfolders. For example, run it in `~/code`. todoer does not look
into a project, a hidden folder or a linked worktree.

todoer reads the todos of the branch that is checked out in each project. Above the todos of a
project, it shows the path of the project relative to the current folder, and the branch.
`ls --json` adds `project` (the path) and `branch` to each todo.

```
$ cd ~/code && todoer ls
todoer  main
  available    fix-login

work/api  feat/rate-limit
  in-progress  add-rate-limit  me@example.com
```

Only `ls` reads many projects. To use another command, go to the folder of the project.

## Dependencies and labels

| Command                            | Does                                                          |
| ---------------------------------- | ------------------------------------------------------------- |
| `todoer dep add <name> [<name>...] [-l label]...` | the first todo depends on each other todo, and on each label |
| `todoer dep rm <name> [<name>...] [-l label]...`  | removes those dependencies                     |
| `todoer dep show <name>`           | prints the tree of what the todo waits for, and what needs it |
| `todoer label add <name> <label>`  | adds a label to a todo                                        |
| `todoer label rm <name> <label>`   | removes a label from a todo                                   |
| `todoer labels [--json]`           | lists the labels with the number of todos and their rules     |
| `todoer labels rm <label>`         | removes the label from each todo, and removes its rule        |
| `todoer labels mv <old> <new>`     | renames the label in each todo, and renames its rule          |
| `todoer labels rule <label> [text]` | writes the rule of the label; the text can come on stdin     |
| `todoer labels rule <label> --rm`  | removes the rule of the label                                 |
| `todoer progress [label] [--json]` | prints the progress of a label, or one line for each label    |

- todoer refuses a dependency that makes a cycle. There is no depth limit.
- `dep show` prints a todo that two paths reach once. The second path says `see above`.

### Depend on a label

A todo can wait for a group of todos: all the todos of a label.

```bash
todoer add new-login-page -D auth-backend-refactor
todoer dep add new-login-page -l auth-backend-refactor
todoer ls
```

```
available    move-tokens
blocked      new-login-page  waits for label auth-backend-refactor
```

- The todo gets the field `dependsOnLabels`. It waits while another todo has the label. A todo that
  gets the label later also blocks it.
- `add -D` and `dep add -l` refuse a label that no other todo has.
- `label add`, `labels mv` and `add` refuse a change that makes a cycle through a label.
- When no other todo has the label, `done`, `pr merge`, `label rm` and `labels rm` remove the label
  from `dependsOnLabels`. `labels mv` renames the label in `dependsOnLabels` too.
- `dep show` prints `label <label>` with the todos of the label below it. `ls --json` lists the
  labels in `dependsOnLabels`, and the todos that the todo waits for in `waitsFor`.
- `claim` lets you claim the todo together with the todos of the label.
- A label matches `^[a-z0-9-]+$`. A label exists while one todo has it, or while it has a rule.
- `labels rm` removes the label from each todo that has it, and removes its rule. Then the label
  does not exist.
- `labels mv` renames the label in each todo that has it, and renames its rule. When a todo
  already has the new label, it keeps one. When the two labels have a rule, `labels mv` stops and
  changes nothing. Then remove one rule first.
- `labels rm`, `labels mv` and `labels rule` do not commit.

### Label rules

A label rule tells in plain words which todos must have a label. It is the file
`.todoer/labels/<label>.md`. todoer does not read the words. The agent reads the rules before
`todoer add`, and gives `-l` for each rule that the new todo matches.

```bash
todoer labels rule auth "Add this label to a todo that changes the login, the sessions or the tokens."
todoer labels
```

```
auth  2 todos
  Add this label to a todo that changes the login, the sessions or the tokens.
docs  0 todos
```

- `labels` prints the rule of a label below the label, with an indent. A label without a rule
  shows only the count. A rule can exist before a todo has its label: then the count is `0 todos`.
- `labels --json` adds the field `rule` to each label: the text of the rule, or `null`.
- `labels rule <label> [text]` writes the rule, or replaces it. When there is no text and stdin is
  not a terminal, it reads the text from stdin.
- `progress` puts each todo of the label in one group: in a pull request (an open pull request
  names it in `todos`), in progress (it has an assignee) or not started. The todos that exist now
  are the total. Each group lists its todos in dependency order. A not started todo shows
  `available`, or what it waits for. A blocking todo without the label shows `(outside label)`.

## Pull requests

| Command                                                                                   | Does                                                  |
| ----------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| `todoer pr submit --branch b -n name [-m text] [--todo name]... [--update] [--no-commit]` | writes the commits of the branch as a pull request on `b` |
| `todoer pr ls [--json]`                                                                   | lists the pull requests of the current checkout       |
| `todoer pr merge <pr> [--no-commit]`                                                      | applies the patch on the current branch               |
| `todoer pr apply <pr>`                                                                    | applies the patch to the working tree, no commit      |
| `todoer pr close <pr> [--no-commit]`                                                      | removes a pull request without a merge                |

A `<pr>` is the name or the path of the folder.

- `submit` takes the commits of the current branch after the merge base with `--branch`. This is
  all the work since the current branch started from `--branch`. Commits that change only
  `.todoer/` do not count.
- `submit` squashes these commits into one commit in `pr.patch`. The author is you. The
  message is `todoer: merge <name>`. The messages of the commits of the work are not used.
- `submit` writes `.todoer/prs/<name>/` on the branch `--branch`, and commits it there. The
  pull request lands on the branch that it goes into, so the review and the merge happen there.
- When `--branch` is checked out in a checkout, for example your main checkout, `submit` writes
  the folder in that checkout and commits it there. When that checkout has other changes that are
  not committed, `submit` does not commit. It writes the folder and prints a warning. Then commit
  the folder yourself.
- When `--branch` is not checked out, `submit` commits the folder on the branch directly. Then
  `--no-commit` is not possible.
- `--branch` is necessary on each submit. It must be a local branch.
- `submit` stops when the current checkout has changes that are not committed. Commit the work
  first.
- The name of the folder is the id and the title of the pull request, as for a todo. `-n` gives
  the name. `-n` is necessary: todoer does not make a name. When `--branch` has an open pull
  request with that name, `submit` stops and names `-n <pr> --update`.
- `--todo` names a todo of the pull request. Give it once for each todo. `pr merge` removes these
  todos.
- A pull request keeps only the patch of its latest submit. `-n <pr> --update` replaces the patch
  and the diff of `<pr>`, and removes the review. git history keeps the old patch and the old
  review. The current branch can have any name: `-n` finds the pull request.
- A submit without `--update` does not change an open pull request. It stops and names
  `-n <pr> --update`. Do what the comments ask first. todoer does not check the comments.
- A pull request does not need the branch of the work after the submit. You can remove that
  branch and its worktree.
- To continue a pull request on a new branch, start the branch at the `base` of `pr.md`, then
  run `git am .todoer/prs/<pr>/pr.patch` from the checkout of `--branch`. This restores the one
  commit of the pull request. The patch always applies
  on `base`. Then update with `-n <pr> --update`. todoer does not make the branch or the worktree.
- `--update` writes the whole pull request again, as for a new one. `-m` replaces the
  description, and `--todo` replaces the todos. Without them, the update copies the old
  description and the old todos, and prints what it copied.
- `ls` lists the pull requests of the current checkout, with the number of comments and the
  author.

### Apply

To run the code of a pull request, or to continue its work, make a new branch from the branch
that holds the pull request. Then run `todoer pr apply <pr>`.

- `apply` puts the changes of the patch in the working tree and in the index. It does not commit.
- `apply` stops when the current checkout has changes that are not committed.
- `apply` first tries the patch on a copy of the git index. When the patch does not apply,
  nothing changes.
- To update the pull request, commit your changes. Then run
  `todoer pr submit --branch <b> -n <pr> --update`. `apply` prints this command. Give `<b>`
  yourself: todoer does not guess the branch.

### Merge

There is no approve step. To accept a pull request, merge it. Run `todoer pr merge <pr>` in the
checkout of the branch that the pull request goes into. `submit` already put the folder there.

- `merge` stops when the review has a comment. A comment is resolved when
  `pr submit --update` removes the review.
- When the changes of the patch are already on the current branch, for example after a git merge
  of the branch, `merge` does not apply the patch again.
- Else `merge` runs `git am --3way`. A base that moved is not a problem while the patch applies.
  When the patch does not apply, the branch does not change. Ask the author to rebase the branch
  and run `todoer pr submit -n <pr> --update`.
- Then `merge` removes the folder and the todos of the pull request, and removes the todos from
  each `dependsOn`. It adds these changes to the commit of the patch, so the merge is one commit.
  When the changes were already on the branch, `merge` commits this as `todoer: merge <pr>`.
- With `--no-commit`, `merge` first tries the patch on a copy of the git index. When it applies,
  `merge` runs `git apply --3way`. The changes stay in the working tree and in the index.
- `close` removes the folder. The todos stay.

### Commits

`claim`, `unclaim`, `submit`, `merge` and `close` commit only their own files, with
`git commit -- <paths>`. `submit` commits on the branch `--branch`. The other commands commit on
the current branch. With `--no-commit`, the changes stay in the working tree. When the checkout has
other changes that are not committed, todoer does not commit. It writes the files and prints a
warning with the other changes. Then commit the files yourself. Files that git does not track do
not count.

## Review

| Command                                                    | Does                                       |
| ---------------------------------------------------------- | ------------------------------------------ |
| `todoer pr summary <pr> [-a area,...] [-x area,...]`       | the files, the comment counts, the areas   |
| `todoer pr details <pr> [file...] [--context N] [--lines 40-80] [-a area,...] [-x area,...]` | the diff with line numbers and comments |
| `todoer pr comment <pr> [target] [text]`                   | adds a comment to `review.md`              |
| `todoer pr review <pr> [-a area,...] [-x area,...]`        | the summary and all the details            |

- Review in the checkout of the branch that the pull request goes into. The folder is there.
- `details` shows the old and the new line numbers. `--context N` reads the old file from the base
  and shows `N` more lines around each change. `--lines 40-80` shows only these new lines.
- Colors and `$PAGER` are on only when the output is a terminal. `NO_COLOR` turns colors off.
- The review, `review.md`, is for the latest patch only. `todoer pr submit --update` removes it,
  and the next review starts with no comments.
- `comment` writes `review.md` and does not commit it. When the review is done, commit it:
  `git add .todoer/prs/<pr>/review.md && git commit -m "Review <pr>" -- .todoer/prs/<pr>/review.md`.
- There is no verdict. To accept the pull request, merge it. To ask for changes, commit the review
  and ask an agent to do the comments and run `pr submit --update`.
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
adds the comments that you ask for with `pr comment`. Then commit the review.

## Code areas

An area is a named part of the code. `.todoer/areas` gives each area its paths:

```
# the last pattern that matches a path wins
security: /src/api/auth
tests: /tests
e2e-tests: /tests/e2e
unit-tests: *.test.js *_test.go
docs: /docs/ *.md
```

- Each line is `<area>: <pattern>...`. Lines that start with `#` are comments.
- The patterns use the gitignore rules. The last pattern that matches a path gives its area.
  Thus put the general areas first and the specific areas after them.
- A pattern with no `/` matches the file name in each folder. `unit-tests: *.test.js` puts
  `src/api/auth/login.test.js` in `unit-tests`, because its line comes after `security`.
- `pr summary` and `pr review` show each area that the pull request changes, with the number of
  files and lines. `(no area)` counts the files that no pattern matches.
- For a renamed file, the areas of the old and the new path count.
- When `.todoer/areas` is missing, the summary shows no areas.
- `-a security,e2e-tests` (`--area`) shows only the files in these areas.
- `-x e2e-tests,unit-tests` (`--exclude`) hides the files in these areas. The files with no area
  stay.
- `summary`, `details` and `review` take `-a` and `-x`. You can use the two together. Give the
  names with commas, or give the option more than one time. An unknown name is an error.
- A renamed file stays when one of its areas is shown.

## Other commands

| Command                   | Does                                               |
| ------------------------- | -------------------------------------------------- |
| `todoer skill [--init]`   | prints the agent workflow, or writes it as a skill |
| `todoer update`           | installs the latest release                        |
| `todoer help`             | prints the help                                    |
| `todoer version`          | prints the version                                 |

`skill --init` writes `.claude/skills/todoer/SKILL.md` and `.agents/skills/todoer/SKILL.md`.
