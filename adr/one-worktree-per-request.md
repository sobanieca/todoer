# todoer start makes one branch and one worktree per request

## Context

An agent must not change the main checkout. The person can ask for one todo or for several todos at
a time.

## Decision

**`todoer start <ids>` makes the branch `todoer/<ids>` and its worktree from the main branch.**

- Under Claude Code the worktree goes to `.claude/worktrees/`, beside the worktrees of Claude Code.
  Otherwise it goes to `.todoer/worktrees/`. `worktreesDir` changes it.
- todoer adds the worktree folder to `.git/info/exclude`.
- `--here` claims the todos for a worktree that another tool made.
- The page offers one worktree for all the selected todos, or one worktree each.

## Consequences

- One pull request can close several todos.
- A todo in progress cannot start again while its worktree exists.
