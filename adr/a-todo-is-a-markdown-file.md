# A todo is a markdown file and its path is the id

## Context

Todos with generated ids in `.todoer/todos/` were far from the code. A person could not move them
next to the code that they are about, and the ids meant nothing.

## Decision

**A todo is a markdown file with a `status` field in its front matter, anywhere in the
repository. Its path from the repository root is its id.**

- `status` is `todo`, `in-progress`, `done` or `dropped`. `assignee` is a git email.
- todoer finds the todos with `git ls-files -co --exclude-standard '*.md'` and skips `.todoer/`.
- A todo is on the code branch. A change to a todo goes through a commit, as for any other file.
- There are no `id`, `created` or `updated` fields. git has the history.
- Images are plain markdown links to files near the todo.

## Consequences

- `todoer mv` must change each `dependsOn` that names the old path.
- `todoer check` can run in CI, because the todos are in the repository.
