# A pull request is a folder with a patch, on the branch that it goes into

## Context

The review must not need the branch of the author. One pull request must add one commit to the
branch that it goes into, not the steps of the work. People and agents share the pull requests
through git. todoer must not know a main branch. The person reviews and merges on their own branch.
The pull request must be there without a cherry-pick or a merge by hand.

A submit from the branch of the work had to write on another branch. When no checkout had that
branch, todoer had to commit with a temporary index. A submit in the checkout of the branch that
the pull request goes into only writes files in its own working tree.

## Decision

**A pull request is the folder `.todoer/prs/<name>/`. In the checkout of the branch that the pull
request goes into, `todoer pr submit --branch <work>` writes `pr.patch` of the commits of `<work>` after the merge
base with the current branch. `pr.patch` holds one commit: a squash of these commits.**

- For the squash, todoer makes a commit with `git commit-tree` on the merge base, with the tree of
  `<work>`. `git format-patch -M -1` of this commit, without the folder of the pull request, is `pr.patch`. No branch
  changes.
- The author of the squash is the person who submits. The message is `todoer: merge <name>` (see
  [the-merge-message-is-the-pull-request-name](the-merge-message-is-the-pull-request-name.md)).
- The name of the folder is the id and the title, as for a todo. `pr.md` has no `title`. `-n`
  gives the name.
- `pr.md` holds the author and the base. `--branch` and `-n` are necessary on each
  submit (see [a-pull-request-names-no-branch](a-pull-request-names-no-branch.md)).
- `--branch` must be a local branch, and not the current branch.
- todoer writes the folder in the working tree of the current checkout, and does not commit it
  (see [todoer-never-commits](todoer-never-commits.md)).
- Every git command of a submit excludes the folder of the pull request. The patch carries the
  other changes of `.todoer/` (see
  [a-pull-request-carries-its-todo-changes](a-pull-request-carries-its-todo-changes.md)).
- When `<work>` is checked out in a worktree, a submit stops while that worktree has changes that
  are not committed.
- `-n <pr> --update` writes the pull request again (see
  [an-update-replaces-the-pull-request](an-update-replaces-the-pull-request.md)).
- `todoer pr apply <pr>` puts the patch in the working tree of the current branch (see
  [apply-a-pull-request-to-the-working-tree](apply-a-pull-request-to-the-working-tree.md)).
- `todoer pr apply --merge` works on the current branch. When `git apply --reverse --check` shows
  that the changes are already there, it does not apply the patch. Else it tries the patch on a
  copy of the git index, then runs `git apply --3way`. A conflict changes nothing. Then it removes
  the folder and the todos of the pull request, and stages all these changes.
- `todoer pr close` removes the folder.

## Consequences

- The review commands read only the diff in `pr.patch`. They need no worktree, so agents can use
  them.
- The pull request has one copy of the change. The review and the merge read the same file.
- The submit, the review and the merge happen in the checkout of the branch that the pull request
  goes into. An agent in a worktree runs the submit in the checkout of the person.
- To submit into a branch, check it out first.
- A merge commit holds the changes of the pull request, and the removal of its folder and its
  todos. The commits of the work stay only on the branch of the work.
- A base that moved is not a problem while the patch applies with a three-way merge. On a conflict,
  a person runs `pr apply <pr>` on a new branch, resolves the conflicts, commits and runs
  `pr submit --branch <new branch> -n <pr> --update`.
- A pull request from a branch of a feature branch goes into the feature branch. The main branch
  never sees it.
- git history keeps the folder of a merged or a closed pull request, after the person commits it.
