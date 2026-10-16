# A todo is a file in the todoer folder, and its name is the id

## Context

A todo with a `status` field could be any markdown file in the repository. todoer had to read every
markdown file to find the todos, and a done todo stayed in the repository. A path as an id was long,
and a move changed it.

## Decision

**A todo is a file `.todoer/todos/<name>.md` in the current checkout. The name is the id and the
title. The file exists only while the work is open.**

- The name matches `^[a-z0-9-]+$`. The folder has no subfolders. Labels collect the todos.
- The front matter is optional. It has `assignee`, `labels`, `dependsOn` and `dependsOnLabels`.
  There is no `title` and
  no `status` field.
- A todo with an assignee is in progress. A todo without an assignee is `available` or `blocked`.
- `todoer done` and `todoer pr merge` remove the file. There is no `dropped` status: a dropped todo
  is also removed. git history keeps the file.
- todoer reads `.todoer/todos/` with `read_dir`. It reads no other markdown file.
- Todo commands change the files of the current checkout. They do not commit. A feature branch can
  hold todos that are not on another branch.
- Images are plain markdown links.

## Consequences

- `todoer mv` must change each `dependsOn` that names the old name.
- `todoer done` removes the name from each `dependsOn`, so a missing name in `dependsOn` is always
  a problem for `todoer check`.
- `todoer check` can run in CI, because the todos are in the repository.
- Progress counts the todos with an assignee. A done todo is gone, so it does not count.
