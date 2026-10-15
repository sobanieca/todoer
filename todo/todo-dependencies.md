---
title: Dependencies between todos
status: done
labels: [cli-reshape]
dependsOn: [todo/migrate-to-rust.md]
---

`dependsOn` in a todo lists the paths of other todos. A todo waits until each todo in its list is
`done` or `dropped`.

- A todo cannot depend on a pull request. The pull request sets its todo to `done`, so a
  dependency on the todo waits for the merge.
- There is no depth limit. todoer refuses a dependency that makes a cycle.
- A todo that waits is `blocked`. A `todo` that does not wait is `ready`.

## Commands

| Command                              | Does                                                    |
| ------------------------------------ | ------------------------------------------------------- |
| `todoer dep add <path> <path> [...]` | the first todo depends on each other todo               |
| `todoer dep rm <path> <path> [...]`  | removes those dependencies                              |
| `todoer dep show <path>`             | prints the tree of what the todo waits for, and what needs it |

`dep show` prints a todo that two paths reach once. The second path says `see above`.

## ADRs

Replace `dependencies-between-items`.
