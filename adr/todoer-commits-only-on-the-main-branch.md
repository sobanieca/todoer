# todoer commits only on the main branch, in the main checkout

## Context

Many agents work in parallel worktrees. Each one must see at once which pull requests are open. A
commit on the main branch is visible to every worktree.

A todo can belong to a feature branch and not be on the main branch yet. A person also wants to
decide when a change of a todo goes into git.

## Decision

**Pull requests, merges, closes and verdicts are commits on the main branch, in the main checkout.
Todo commands, also `claim`, change the files of the current checkout and do not commit.
`.todoer/` is in git.**

- Before each commit, todoer checks that the main checkout is on the main branch. When it is on a
  feature branch, todoer stops.
- todoer commits only the paths of the change: `git commit -- <paths>`. Other staged changes stay.
- A lock file `todoer.lock` in the git common folder lets one todoer command write at a time. A
  command holds the lock from the read that decides the change until the write or the commit is
  done.
- A todo command works on any branch. The person commits the change on the branch of their choice.
- `claim` refuses a todo of another person. `--force` only skips the dependency check.
- The main branch is `mainBranch` in `todoer.json`, else `origin/HEAD`, else `main`, else `master`.

## Consequences

- A person must keep the main branch checked out in the main checkout.
- Agents see the claims of the other agents only when they claim in the same checkout. Two
  checkouts can claim the same todo until the person commits and the other checkout pulls.
- todoer does not push. See `todo/sync-with-remote.md`.
