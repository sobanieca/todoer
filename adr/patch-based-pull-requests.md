# A pull request is a folder of patches on the main branch

## Context

The review must not need the branch of the author, and the merge must keep the commits of the
author. People and agents share the pull requests through git.

## Decision

**A pull request is the folder `.todoer/prs/<slug>/` on the main branch. `todoer pr submit` writes
`pr.patch` (`git format-patch`) and `pr.diff` (`git diff -M`), and commits them.**

- `pr.md` holds the title, the author, the branch, the base and the todos.
- The base is the merge base of the branch and the main branch.
- Every git command of a submit excludes `.todoer/`.
- A second submit from the same branch replaces the patch and the diff. There are no revisions.
  git history keeps the old patches.
- `todoer pr merge` runs `git am --3way` on the main branch, and `git am --abort` on a failure.
  Then it sets the todos to `done` and removes the folder, in one commit.
- `todoer pr close` removes the folder in one commit.

## Consequences

- The review commands read only `pr.diff`. They need no worktree, so agents can use them.
- The merge keeps the author and the message of each commit, with new commit ids.
- git history keeps the folder of a merged or a closed pull request.
- todoer cannot show the changes between two submits. Use git history for that.
