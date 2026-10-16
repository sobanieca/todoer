# todoer works on the current branch and knows no main branch

## Context

todoer committed every pull request, every merge and every verdict on the main branch, in the main
checkout. The person had to keep the main branch checked out there, and `todoer.json` had to name
the main branch. A person who works on a feature branch, with agents on branches of that feature
branch, could not use todoer there. A person also wants to decide when a change goes into git.
But a claim is only useful when other people see it, so it must be ready to push at once.

## Decision

**todoer reads and writes the `.todoer/` folder of the current checkout, on any branch. It has no
main branch, no main checkout and no settings file. `claim`, `unclaim` and the pull request
commands commit only their own files. The other todo commands do not commit.**

- `claim` and `unclaim` commit the todo files on the current branch. `pr merge` and `pr close`
  commit on the current branch. `pr submit` commits on the branch that the pull request goes into.
- Each commit has only the paths of the change: `git commit -- <paths>`. Other staged changes stay.
- `--no-commit` keeps the changes in the working tree.
- When the checkout has other changes that are not committed, todoer does not commit. It writes the
  files and prints a warning with the other changes. The person commits. Files that git does not
  track do not count.
- The other git commands read: the diff and the patch of a submit, and the old file for a review.
  `pr merge` also applies the patch. todoer never pushes or fetches.
- A lock file `todoer.lock` in the git common folder lets one todoer command write at a time.
- `claim` refuses a todo of another person. `--force` only skips the dependency check.
- Outside a git repository, `todoer ls` reads each project folder in the current folder, on the
  branch that is checked out there, and warns about this branch.

## Consequences

- Other people see a claim, a pull request or a review only after a push and a pull.
- A pull request can go into any branch, also a feature branch. The pull requests of a feature
  branch stay on that branch.
