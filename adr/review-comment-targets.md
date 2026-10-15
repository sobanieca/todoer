# A review comment names lines, a file, a folder or the pull request

## Context

A person often wants a different name or a different place for a file or for a whole folder. A line
comment cannot say that well. Some comments are about a block of lines, or about the whole change.

## Decision

**A comment target is `path:line`, `path:start-end`, `path:old:line`, `path`, `folder/`, or no
target for the whole pull request.**

- `path:line` and `path:start-end` are lines of the new version. `path:old:line` is a removed line.
- `path:line` can name any line of the new version of any file: a file of the pull request, or a
  file of the base that the pull request does not change. A reviewer can point to code that must
  change too.
- `path:old:line` must be a line of the diff.
- A file target or a folder target must be in the pull request or in its base.
- A line comment keeps the text of its lines, so an agent sees it without the diff.
- The review is one file, `review.md`, in markdown. See `the-review-is-for-the-latest-patch`.
- A new comment is a draft: a change in the review file that is not committed. `approve` and
  `request-changes` commit the review file, so each review is one commit.

## Consequences

- An agent reads every target type with `todoer pr details`. It shows a line outside the changes
  with two lines around it, in line order with the hunks.
- `pr summary` lists a file that the pull request does not change when a comment names it.
