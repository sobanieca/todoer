# Labels collect related todos

## Context

Groups were names on todos, with their own commands. A label in the front matter is the same idea,
and every markdown tool can show it.

## Decision

**A label is a name in the `labels` list of a todo. It matches `^[a-z0-9-]+$`.**

- A label exists while one todo has it, or while it has a rule (see
  [label-rules](label-rules.md)). A pull request has the labels of its todos.
- `todoer progress <label>` puts the todos with the label that exist now in three groups: in
  progress, in a pull request and not started. It lists the todos in dependency order, and marks
  a blocking todo without the label.

## Consequences

- Lowercase only, so `Rust` and `rust` cannot be two labels.
