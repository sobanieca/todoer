# The merge message is the name of the pull request

## Context

The squash in `pr.patch` used the message of the work commits. For more commits, the subject of the
first commit was the subject, and the body listed the subject of each commit. An update from a new
branch, or after `pr apply`, starts with other commits. Then the message changed to the first new
commit, for example `WIP`, and the old message was lost. The name of the pull request is stable,
and the person gives it with `-n`.

## Decision

**The message of the squash in `pr.patch` is `todoer: merge <name>`. todoer does not use the
messages of the work commits.**

- `pr apply --merge` does not commit (see [todoer-never-commits](todoer-never-commits.md)). It
  prints `git commit -m "todoer: merge <name>"`, so the person commits the merge with this
  message.
- `git am` of `pr.patch` makes a commit with this message.
- The description of `pr.md` is not in the message.

## Consequences

- When the person uses the printed command, the git log of a branch shows one
  `todoer: merge <name>` commit for each merged pull request.
- An update by any person, from any branch, does not change the message.
- To read the description, read `pr.md` in the parent of the merge commit.
- The author of the merge commit is the person who commits it. The author of the latest submit is
  in `pr.md`, in the parent of the merge commit.
