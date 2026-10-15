---
title: Labels and the progress of a label
status: todo
labels: [cli-reshape]
dependsOn: [todo/todos-as-files.md, todo/todo-dependencies.md]
---

A label is a name in the `labels` list of a todo. A todo can have many labels, and a label can be
on many todos. A label exists while one todo has it.

- A label must match `^[a-z0-9-]+$`. Lowercase only, so `Rust` and `rust` are not two labels.
- A pull request has the labels of its todos.

## Commands

| Command                            | Does                                                |
| ---------------------------------- | --------------------------------------------------- |
| `todoer label add <path> <label>`  | adds a label to a todo                              |
| `todoer label rm <path> <label>`   | removes a label from a todo                         |
| `todoer labels`                    | lists the labels with the number of todos           |
| `todoer progress [label] [--json]` | prints the progress of a label, or one line for each label |

## Progress

```
migration-to-rust-followup   7/12 done (58%)  [#######-----]

in progress (2)
  todo/rust/port-cli.md        @sobanieca  pr: port-cli v2, changes requested
  todo/rust/port-config.md     @agent-1
ready (1)
  todo/rust/remove-node-build.md
blocked (2)
  todo/rust/drop-js-tests.md   waits for todo/rust/port-cli.md
  todo/rust/release.md         waits for todo/infra/ci-image.md  (outside label)
```

- A `dropped` todo is not in the total.
- The open todos are in dependency order, so `ready` shows the work that can start now.
- A todo outside the label that blocks a todo in the label shows with `(outside label)`.
