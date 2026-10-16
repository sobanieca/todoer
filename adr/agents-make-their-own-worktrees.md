# Agents make their own worktrees

## Context

Agent tools make worktrees in their own way, for example the `EnterWorktree` tool of Claude Code.
These tools also prepare the worktree, for example they copy the files of `.worktreeinclude` that
git does not track. `todoer start` made a second worktree and a branch name that the tool did not
know.

## Decision

**todoer does not make worktrees or branches. An agent makes them with its own tool or with
`git worktree add`. This is true for new work and for the fix of a review.**

- `todoer pr submit --branch <branch>` runs in any checkout, on any branch that is not `<branch>`.
- For new work, the worktree starts from the branch of the person, so `--branch` is that branch.
- A pull request does not need the branch of the work after the submit. The person can remove the
  branch and its worktree.
- To fix a review, the agent makes a new worktree and branch at the `base` of `pr.md`, and restores
  the commit with `git am .todoer/prs/<pr>/pr.patch`. The patch always applies on `base`. Then
  `pr submit -n <pr> --update` continues the pull request from the new branch.
- There is no `pr restore` or `pr checkout`. These commands would make a branch or a worktree
  without the preparation of the agent tool.

## Consequences

- `merge` does not remove the worktree of the author.
- The skill tells the agent the two git steps of a restore. When the tool starts the worktree at
  another commit, the agent runs `git reset --hard <base>` first. Files that git does not track
  stay.
