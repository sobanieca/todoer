# Todos depend on todos, with no cycle

## Context

Some work cannot start before other work is done. A plan of many steps needs a dependency graph
with no depth limit.

## Decision

**`dependsOn` in a todo lists the paths of other todos. A todo waits until each of them is `done`
or `dropped`.**

- A todo cannot depend on a pull request. A merge sets its todos to `done`, so a dependency on the
  todo waits for the merge.
- todoer refuses a dependency that makes a cycle. `todoer check` reports a cycle that a hand edit
  made.
- A `todo` that waits is `blocked`. A `todo` that does not wait is `ready`.
- `claim` refuses a todo that waits, unless `--force`.

## Consequences

- `dep show` prints a todo that two paths reach once. The second path says `see above`.
