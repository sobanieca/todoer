# A person reviews the patch by talking with an agent

## Context

A review worktree needed a staged patch, code comments that were not committed, a marker file and
checks for an old patch or a changed index. This was much work for a small gain. An agent can read
the patch and add the comments that a person asks for.

## Decision

**A person reviews a pull request by talking with an agent. The agent reads `pr.diff` with
`todoer pr details`, and adds each comment with `todoer pr comment <pr> <target> <text>`.**

- todoer does not make review worktrees. There is no `pr checkout` and no `worktreesDir`.
- Every review command needs the pull request as an argument.
- A person can also review in the terminal with `todoer pr review <pr>`.
- The review happens in the checkout of the branch that the pull request goes into. The person
  commits the review when it is done.

## Consequences

- The review needs no branch and no worktree. Only `.todoer/prs/<name>/` is necessary.
- To run the code of a pull request, apply `pr.patch` with `git am` on a branch of your own.
