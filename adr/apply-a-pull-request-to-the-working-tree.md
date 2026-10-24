# Apply a pull request to the working tree, or merge it

## Context

A person sees a pull request on their branch. They want to run its code, or to change it and
update it. The author can be another person. `git am` of `pr.patch` at `base` needs the base, the
path of the patch and a branch at the base. It also makes a commit that the person does not want.
A branch at `base` also does not have the folder of the pull request. todoer must not make
branches or worktrees (see [agents-make-their-own-worktrees](agents-make-their-own-worktrees.md)).
There must be one way to continue a pull request, also when its branch moved.

A merge also applies the patch. Two commands that apply the same patch were one command too many.

## Decision

**`todoer pr apply <pr>` applies `pr.patch` to the working tree and the index of the current
checkout. It does not commit, and it does not make a branch or a worktree. `todoer pr apply <pr>
--merge` merges the pull request and closes it.**

- Without `--merge`, `apply` is for follow-up work. The pull request stays. `apply` removes the
  todos of the pull request, as the patch does.
- The person makes the branch or the worktree first, from the branch that holds the pull request.
  This is the one way to continue a pull request, for a person and for an agent.
- Without `--merge`, `apply` stops when the checkout has changes that are not committed. Files in
  the folder of the pull request and files that git does not track do not count.
- Without `--merge`, `apply` runs `git apply --3way`. On a conflict, it writes conflict markers
  and names the files. The person resolves them and commits. When the patch does not apply for
  another reason, nothing changes.
- When the changes are already in the checkout, `apply` says so and only removes the todos of the
  pull request.
- `apply` prints the next command: `todoer pr submit --branch <current branch> -n <pr> --update`.
  The person commits the changes, checks out the branch that the pull request goes into, and runs
  it there.
- `--merge` stops on a comment, applies the patch with `git apply --3way`, removes the folder and
  the todos of the pull request, and stages these changes (see
  [patch-based-pull-requests](patch-based-pull-requests.md)). It does not commit. It tries the
  patch on a copy of the git index first. On a conflict, nothing changes, and the error names the
  steps of an update with `apply`.

## Consequences

- Any person can continue a pull request. The commits of the update replace the patch.
- The `base` of the update is the merge base of the new branch with the branch that the pull
  request goes into. This is usually a newer commit than the old base. It is always on that
  branch.
- The messages of the new commits do not change the merge message (see
  [the-merge-message-is-the-pull-request-name](the-merge-message-is-the-pull-request-name.md)).
- When the branch moved and the patch conflicts, any person can update the pull request with
  `apply` on a new branch. The author does not need to rebase.
- There is no `pr merge` command. A merge is `pr apply --merge`.
