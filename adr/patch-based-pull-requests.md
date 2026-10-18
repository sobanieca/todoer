# A pull request is a folder with a patch, on the branch that it goes into

## Context

The review must not need the branch of the author. One pull request must add one commit to the
branch that it goes into, not the steps of the work. People and agents share the pull requests
through git. todoer must not know a main branch. The person reviews and merges on their own branch.
The pull request must be there without a cherry-pick or a merge by hand.

## Decision

**A pull request is the folder `.todoer/prs/<name>/`. `todoer pr submit --branch <b>` writes
`pr.patch` and `pr.diff` (`git diff -M`) of the commits after the merge base with `<b>`, and
commits the folder on `<b>`. `pr.patch` holds one commit: a squash of these commits.**

- For the squash, todoer makes a commit with `git commit-tree` on the merge base, with the tree of
  `HEAD`. `git format-patch -1` of this commit, without `.todoer/`, is `pr.patch`. No branch
  changes.
- The author of the squash is the person who submits. The message is `todoer: merge <name>` (see
  [the-merge-message-is-the-pull-request-name](the-merge-message-is-the-pull-request-name.md)).
- The name of the folder is the id and the title, as for a todo. `pr.md` has no `title`. `-n`
  gives the name.
- `pr.md` holds the author, the base and the todos. `--branch` and `-n` are necessary on each
  submit (see [a-pull-request-names-no-branch](a-pull-request-names-no-branch.md)).
- When `<b>` is checked out in a checkout, todoer writes the folder there and commits it with
  `git commit -- <folder>`. When that checkout has other changes, todoer does not commit.
- When `<b>` is not checked out, todoer commits the folder on `<b>` with a temporary index,
  `git commit-tree` and `git update-ref`. The working trees do not change.
- Every git command of a submit excludes `.todoer/`, so the patch never changes a todo or a pull
  request.
- A submit stops when the checkout of the work has changes that are not committed.
- `-n <pr> --update` writes the pull request again (see
  [an-update-replaces-the-pull-request](an-update-replaces-the-pull-request.md)).
- `todoer pr apply <pr>` puts the patch in the working tree of the current branch, with no commit
  (see [apply-a-pull-request-to-the-working-tree](apply-a-pull-request-to-the-working-tree.md)).
- `todoer pr merge` works on the current branch. When `git apply --reverse --check` shows that the
  changes are already there, it does not apply the patch. Else it runs `git am --3way`, and
  `git am --abort` on a failure. Then it removes the folder and the todos of the pull request, and
  amends the commit of the patch with these changes. When the changes are already there, it
  commits the removal as `todoer: merge <name>`.
- `todoer pr merge --no-commit` tries the patch on a copy of the git index first. Only when the
  patch applies with no conflict, it runs `git apply --3way`. A conflict changes nothing.
- `todoer pr close` removes the folder.

## Consequences

- The review commands read only `pr.diff`. They need no worktree, so agents can use them.
- The review and the merge happen in the checkout of the branch that the pull request goes into.
- A merge adds one commit: the changes of the pull request, and the removal of its folder and its
  todos. The commits of the work stay only on the branch of the work.
- A base that moved is not a problem while the patch applies with a three-way merge. On a conflict,
  the author rebases and runs `pr submit -n <pr> --update`.
- A pull request from a branch of a feature branch goes into the feature branch. The main branch
  never sees it.
- git history keeps the folder of a merged or a closed pull request.
