# Few commands

## Context

todoer had 31 commands. Many did one part of a job that another command also did. `pr review`
printed the output of `pr summary` and `pr details`. `pr merge` and `pr apply` applied the same
patch. `dep add`, `dep rm`, `label add` and `label rm` changed the same front matter as `add`.
`dep show` printed data that `show` also prints. `labels` and `progress` both listed each label.
People and agents had to learn too many names.

## Decision

**One command does one job. The options of a command select the part of the job.**

- `todoer show <name>` prints all the data of a todo. There is no `dep show`.
- Todos have no labels and no dependencies (see
  [a-pull-request-carries-its-todo-changes](a-pull-request-carries-its-todo-changes.md)). There is
  no `edit`, no `labels`, no `progress` and no `check`.
- Todos have no assignee (see
  [a-todo-is-a-file-in-the-todoer-folder](a-todo-is-a-file-in-the-todoer-folder.md)). There is no
  `claim` and no `unclaim`.
- `todoer pr show <pr>` prints the summary and the diff. `--summary` prints only the summary. With
  files, it prints only their diff. There is no `pr summary`, `pr details` or `pr review`.
- `todoer pr apply <pr> --merge` merges. There is no `pr merge` (see
  [apply-a-pull-request-to-the-working-tree](apply-a-pull-request-to-the-working-tree.md)).

## Consequences

- todoer has 15 commands.
- `ls` and `show` print a warning for a bad todo name.
