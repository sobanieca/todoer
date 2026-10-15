---
status: todo
---

A todo is a file in `.todoer/todos/`. The file name is the id and the title. The file exists only
while the work is open. A todo with an assignee is in progress. A todo command changes the files of
the current checkout. It does not commit and does not push. The person decides when to commit, and
on which branch.

A feature branch can hold todos that are not on the main branch. todoer reads the todos of the
branch that is checked out.

## Files

```
.todoer/todos/<name>.md
```

```markdown
---
assignee: agent-1@example.com
labels: [auth]
dependsOn: [fix-the-login-crash]
---

What to do and how to check it.
```

- The name must match `^[a-z0-9-]+$`. The folder has no subfolders. Labels group the todos.
- The name is the id and the title. There is no `title` and no `status` field.
- The front matter is optional. A file with no front matter is a new todo.
- `dependsOn` lists names, not paths.
- Each `*.md` file in `.todoer/todos/` is a todo. todoer reads no other markdown file.

## Commands

Every todo command changes `.todoer/todos/` of the current checkout, on any branch. It does not
commit and does not push. It takes the lock while it reads and writes the files.

| Command                                         | Does                                                  |
| ----------------------------------------------- | ----------------------------------------------------- |
| `todoer add <name> [-l label] [-d name] [text]` | adds the todo; the text can come on stdin             |
| `todoer done <name>...`                         | removes the todos, and removes them from each `dependsOn` |
| `todoer mv <old> <new>`                         | renames a todo and changes each `dependsOn` to it     |
| `todoer claim <name> [--force]`                 | sets the assignee                                     |
| `todoer unclaim <name>`                         | removes the assignee                                  |
| `todoer dep add/rm`, `todoer label add/rm`      | as now, with names                                    |

- `add` and `mv` refuse a bad name, and print a correct name. Example:
  `"Fix_Login" is not a todo name - use fix-login`.
- `add` has no `-t` and no `-p`.
- `done` removes the name from the `dependsOn` of each other todo. Thus a missing name in
  `dependsOn` stays a problem for `check`.
- There is no `dropped`. To drop a todo, run `todoer done`.
- `claim` refuses a todo with an assignee. It accepts a todo with uncommitted changes.
- Other people see a claim only in the same checkout, or after the person commits it and they pull.
- `pr merge` removes the todos of the pull request that are on the main branch, in the merge
  commit. `pr.md` lists names.
- A person can change the text of a todo by hand. The skill tells an agent to use only the
  commands, and to commit the todos only when the person asks.

## States

| State         | Means                                            |
| ------------- | ------------------------------------------------ |
| `in-progress` | the todo has an assignee                         |
| `ready`       | no assignee, and each `dependsOn` todo is gone   |
| `blocked`     | no assignee, and a `dependsOn` todo still exists |

- `ls --all` goes. `ls --json` has `name`, `state`, `assignee`, `labels` and `dependsOn`.
- A todo with an assignee and an open dependency is `in-progress`. `claim --force` makes this.

## Progress

`todoer progress <label>` shows how many todos of the label have an assignee.

```
auth  2/5 in progress (40%)  [##---]

in progress (2)
  fix-the-login-crash  agent-1@example.com
  rate-limit           me@example.com
ready (2)
  remember-me
  logout-everywhere
blocked (1)
  audit-log            waits for log-store  (outside label)
```

- The total is the number of todos with the label that exist now.
- `progress --json` has `total` and `inProgress` in place of `done`.

## Rules to change

- `claim` and `unclaim` do not use `Project::commit_on_main`. They do not check the branch of the
  main checkout.
- A patch never contains `.todoer/`, so a todoer pull request cannot add or change a todo. A
  feature branch brings its todos to the main branch with a git merge.
- `Todos::load` reads `.todoer/todos/` of the current checkout with `read_dir`. No `git ls-files`.

## Done when

- The commands work and have tests.
- `src/help.txt`, `docs/cli.md`, `docs/data.md`, `src/templates/SKILL.md`, the README and
  `AGENTS.md` show the new model.
- The ADRs `a-todo-is-a-markdown-file`, `dependencies-between-todos` and `labels-replace-groups`
  describe the new model.
- The todos of this repository move from `todo/` to `.todoer/todos/`. The done todos are gone.
