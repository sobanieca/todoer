# Todos depend on todos and on labels, with no cycle

## Context

Some work cannot start before other work is done. A plan of many steps needs a dependency graph
with no depth limit. Sometimes work waits for a group of todos, for example a new login page waits
for all the todos of the label `auth-backend-refactor`. The group can get more todos later.

## Decision

**`dependsOn` in a todo lists the names of other todos. A todo waits while a todo of the list
exists.**

- `dependsOnLabels` in a todo lists labels. A todo waits while another todo has a label of the
  list. todoer reads the label each time, so a new todo with the label also blocks the todo.
- A label dependency is an edge to each other todo of the label. A todo can have the label that it
  depends on: then it waits for the other todos of the label.
- When no other todo has the label, `done`, `pr merge`, `label rm` and `labels rm` remove the label
  from `dependsOnLabels`. `labels mv` renames it. Thus a label in `dependsOnLabels` that no other
  todo has is a problem for `todoer check`, as a missing name in `dependsOn` is.
- `add -D` and `dep add -l` refuse a label that no other todo has.
- A todo cannot depend on a pull request. A merge removes its todos, so a dependency on the todo
  waits for the merge.
- todoer refuses a dependency that makes a cycle. A new label on a todo, a new todo and a renamed
  label can also make a cycle through `dependsOnLabels`, so `add`, `label add` and `labels mv`
  refuse it too. `todoer check` reports a cycle that a hand edit made.
- A todo without an assignee that waits is `blocked`. One that does not wait is `available`.
- `claim` refuses a todo that waits, unless `--force`.

## Consequences

- `dep show` prints a todo that two paths reach once. The second path says `see above`.
- `dep show` prints a label dependency as `label <label>`, with the todos of the label below it.
- `ls` shows a label dependency as `waits for label <label>`. `ls --json` lists the todos in
  `waitsFor`, and the labels in `dependsOnLabels`.
