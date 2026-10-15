---
title: A todo is a markdown file and its path is the id
status: todo
labels: [cli-reshape]
dependsOn: [todo/migrate-to-rust.md]
---

A todo is a markdown file in the repository. Its front matter has a `status` field. Its id is its
path relative to the repository root.

```markdown
---
title: Port the CLI parser
status: in-progress
assignee: sobanieca
labels: [migration-to-rust-followup, cli]
dependsOn: [todo/rust/port-config.md]
---

The description.
```

- `status` is `todo`, `in-progress`, `done` or `dropped`.
- Remove the `id`, `created`, `updated`, `branch`, `worktree` and `pr` fields. git has the dates.
- Find the todos with `git ls-files -co --exclude-standard '*.md'`. Keep the files with a `status`
  field in the front matter.
- A command accepts a path relative to the current folder. todoer stores paths relative to the
  repository root.
- Images are plain markdown links to files near the todo. Remove `attach` and `maxImageKb`.
- A todo is on the code branch. A pull request can set `status: done` on its todo, so the merge
  closes the todo.

## Commands

| Command                                          | Does                                           |
| ------------------------------------------------ | ---------------------------------------------- |
| `todoer ls [--all] [--mine] [--label l] [--ready]` | lists the active todos                       |
| `todoer add -t <title> [-p dir] [-l label] [-d path] [text]` | adds a todo and prints its path    |
| `todoer done <path>`                             | sets `status: done`                            |
| `todoer mv <old> <new>`                          | moves a todo and changes each `dependsOn` to it |
| `todoer check`                                   | reports bad todos, see below                   |

- `add` reads the text from stdin when stdin is not a terminal. The file name is a slug of the
  title. `-p` sets the folder, the default is the current folder.
- `--mine` compares `assignee` with the person of `git config user.email`.
- `check` reports an unknown status, a bad label, a missing `dependsOn` path and a cycle. It exits
  with `1` on an error, so CI can run it. Every command prints the same warnings when it reads a
  bad todo.

## ADRs

Replace `plain-files-in-the-main-checkout`. Delete `groups-are-names-on-todos` and
`image-size-limit`.
