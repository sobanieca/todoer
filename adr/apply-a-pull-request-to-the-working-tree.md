# Apply a pull request to the working tree

## Context

A person sees a pull request on their branch. They want to run its code, or to change it and
update it. The author can be another person. `git am` of `pr.patch` at `base` needs the base, the
path of the patch and a branch at the base. It also makes a commit that the person does not want.
todoer must not make branches or worktrees (see
[agents-make-their-own-worktrees](agents-make-their-own-worktrees.md)).

## Decision

**`todoer pr apply <pr>` applies `pr.patch` to the working tree and the index of the current
checkout. It does not commit, and it does not make a branch or a worktree.**

- The person makes the branch or the worktree first, for example from the branch that holds the
  pull request.
- `apply` stops when the checkout has changes that are not committed. Files in `.todoer/` and files
  that git does not track do not count.
- `apply` tries the patch on a copy of the git index first, as `pr merge --no-commit` does. On a
  conflict, nothing changes.
- When the changes are already in the checkout, `apply` says so and changes nothing.
- `apply` prints the next command: `todoer pr submit --branch <b> -n <pr> --update`. The person
  commits the changes and gives `<b>`.

## Consequences

- Any person can continue a pull request. The commits of the update replace the patch.
- The `base` of the update is the merge base of the new branch with `--branch`. This is usually a
  newer commit than the old base. It is always on `--branch`.
- The messages of the new commits do not change the merge message (see
  [the-merge-message-is-the-pull-request-name](the-merge-message-is-the-pull-request-name.md)).
- When the branch moved and the patch does not apply, the author updates the pull request first.
