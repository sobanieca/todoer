# Todos, pull requests and reviews are plain files in the main checkout

## Context

Agents and people must read the todos without an API. The data must work offline. A project can have
many worktrees at the same time.

## Decision

**todoer keeps all its data in `.todoer/` of the main checkout, as markdown and patch files.**

- Every command finds the main checkout with `git worktree list` and uses its `.todoer/`.
- The status and the links of a todo are front matter fields, not folders.
- The ids are 4 random characters, so two people can add todos offline without a collision.
- todoer does not commit `.todoer/`. The person decides to commit it or to ignore it.

## Consequences

- A command in a worktree changes the files of the main checkout.
- `.todoer/` in a worktree is a stale copy. todoer never reads it.
- Sync between machines is plain git, when the person commits `.todoer/`. An orphan branch for the
  data is planned, see `todo/orphan-branch-sync.md`.
