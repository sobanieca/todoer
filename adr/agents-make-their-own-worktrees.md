# Agents make their own worktrees

## Context

Agent tools make worktrees in their own way, for example the `EnterWorktree` tool of Claude Code.
These tools also prepare the worktree, for example they copy the files of `.worktreeinclude` that
git does not track. `todoer start` made a second worktree and a branch name that the tool did not
know.

## Decision

**todoer does not make worktrees or branches. An agent makes them with its own tool or with
`git worktree add`. This is true for new work and for the fix of a review.**

- `todoer pr submit --branch <work>` runs in the checkout of the branch that the pull request goes
  into. `<work>` is the branch of the agent. The agent runs the submit in the checkout of the
  person, not in its worktree.
- For new work, the worktree starts from the branch of the person.
- A pull request does not need the branch of the work after the submit. The person can remove the
  branch and its worktree.
- To fix a review, the agent makes a new worktree and branch from the branch of the person, and
  runs `todoer pr apply <pr>` there. It resolves the conflicts, if any, and commits. Then
  `pr submit --branch <new branch> -n <pr> --update`, in the checkout of the person, continues the
  pull request from the new branch.
- There is no `pr restore` or `pr checkout`. These commands would make a branch or a worktree
  without the preparation of the agent tool. `pr apply` only changes the working tree of the
  current checkout (see
  [apply-a-pull-request-to-the-working-tree](apply-a-pull-request-to-the-working-tree.md)).

## Consequences

- A merge does not remove the worktree of the author.
- The worktree for a fix starts at the same commit as the worktree for new work. The pull request
  and its review must be committed on the branch of the person, so that the worktree has them.
