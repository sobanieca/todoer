# A review comment names a line, a file or a folder

## Context

A person often wants a different name or a different place for a file or for a whole folder. A line
comment cannot say that well.

## Decision

**A comment target is `path:line`, `path:old:line`, `path` or `folder/`.**

- `path:line` is a line of the new version. `path:old:line` is a removed line.
- A path with no line is the whole file. A path that ends with `/` is the whole folder.
- A line comment keeps the text of its line, so an agent sees it without the diff.
- The review of each revision is one file, `review.v<N>.md`, in markdown.

## Consequences

- An agent reads every target type with one command, `todoer pr show`.
- The page shows folder comments beside the tree of changed files.
- A comment does not move to the next revision. The page shows the old comments in a list.
