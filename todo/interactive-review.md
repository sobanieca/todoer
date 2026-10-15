---
title: Interactive review in the terminal
status: todo
labels: [cli-reshape]
dependsOn: [todo/pr-review-cli.md, todo/code-ownership.md]
---

`todoer pr review <pr>` with the interactive mode opens a full screen view of the pull request.
The person moves a cursor through the changes and adds comments.

## Interactive mode

- `--interactive` on a command, `TODOER_INTERACTIVE=1`, or `"interactive": true` in
  `$XDG_CONFIG_HOME/todoer/config.json`. The default folder is `~/.config/todoer`.
- The flag has priority over the variable, and the variable over the file. `--no-interactive`
  turns it off for one command.
- Do not put the setting in `todoer.json`. Agents use the same repository.
- The mode is always off when stdin or stdout is not a terminal, or when `CLAUDECODE` is set.

| Command                    | Interactive          | Not interactive                       |
| -------------------------- | -------------------- | ------------------------------------- |
| `pr review <pr>`           | the full screen view | prints the summary and all the details |
| `pr comment` with no text  | opens `$EDITOR`      | reads stdin                           |
| `add` with no text         | opens `$EDITOR`      | reads stdin                           |

## The view

- The list of files, with owners and comment counts, beside the diff. One column when the terminal
  is less than 120 columns wide.
- Keys: `j`/`k` line, `n`/`p` file, `]`/`[` hunk, `v` select lines, `c` comment, `o` owner filter,
  `a` approve, `r` request changes, `q` quit.
- `c` on a file header adds a file comment.
- A comment opens `$EDITOR`. todoer leaves the raw mode while the editor runs.
- todoer writes each comment to the draft review at once.
- Use the alternate screen, ANSI codes and the raw mode from `libc`. Read the terminal size again
  on `SIGWINCH`.
- Use the same modules as `pr summary`, `pr details` and `pr comment`.
