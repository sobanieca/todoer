# A todo is a file in the todoer folder, its name is the id, and it has no assignee

## Context

A todo with a `status` field could be any markdown file in the repository. todoer had to read every
markdown file to find the todos, and a done todo stayed in the repository. A path as an id was long,
and a move changed it.

A todo had an `assignee`, and `claim` and `unclaim` set it. AI agents do most of the work. A
person gives the todos to the agents, so the assignee was always the same person. A claim was one
more change to commit. When many people work on one feature, they can keep the list of who does
which todo outside todoer.

## Decision

**A todo is a file `.todoer/todos/<name>.md` in the current checkout. The name is the id and the
title. The file exists only while the work is open.**

- The name matches `^[a-z0-9-]+$`. The folder has no subfolders.
- The file is plain markdown. todoer reads no fields from it. There is no `title`, no `status` and
  no `assignee`.
- There is no `claim` and no `unclaim` command. A todo has no state.
- `todoer done` and `todoer pr apply --merge` remove the file. There is no `dropped` status: a
  dropped todo is also removed. git history keeps the file.
- todoer reads `.todoer/todos/` with `read_dir`. It reads no other markdown file.
- Todo commands change the files of the current checkout. They do not commit. A feature branch can
  hold todos that are not on another branch.
- Images are plain markdown links.

## Consequences

- A todo file does not name another todo. A rename or a removal changes only one file.
- A pull request carries the removal of its todos (see
  [a-pull-request-carries-its-todo-changes](a-pull-request-carries-its-todo-changes.md)).
- todoer does not show who works on a todo. An open pull request that removes the todo shows
  that the work is done and waits for a merge.
