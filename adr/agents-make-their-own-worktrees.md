# Agents make their own worktrees

## Context

Agent tools make worktrees in their own way, for example the `EnterWorktree` tool of Claude Code.
`todoer start` made a second worktree and a branch name that the tool did not know.

## Decision

**todoer does not make work worktrees. An agent claims a todo, then makes a worktree from the main
branch with its own tool or with `git worktree add`.**

- `todoer pr submit` runs in any worktree on a branch that is not the main branch.

## Consequences

- `merge` does not remove the worktree of the author.
