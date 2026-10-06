# A pull request is a patch series plus a combined diff

## Context

The person reviews on the main branch. The review must not need the branch of the agent, and the
merge must keep the commits of the agent.

## Decision

**`todoer pr submit` writes `v<N>.patch` (`git format-patch`) and `v<N>.diff` (`git diff -M`) for
each revision.**

- The base is the merge base of the branch and the main branch.
- Every git command of a submit excludes `.todoer/`.
- A second submit on the same branch adds the next revision to the open pull request.
- `todoer pr merge` runs `git am --3way` on the main branch, and `git am --abort` on a failure.

## Consequences

- The review page reads only `v<N>.diff`. It needs no git command.
- The merge keeps the author and the message of each commit, with new commit ids.
- The merge deletes the branch with `git branch -D`, because `git am` makes new commits.
