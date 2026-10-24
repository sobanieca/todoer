# todoer never commits

## Context

The todo commands and the pull request commands committed their own files. A submit committed the
folder on the branch that the pull request goes into, also when no checkout had that branch. Then
todoer needed a temporary index, `git commit-tree` and `git update-ref`. Each command also had to
skip the commit when the checkout had other changes, and `--no-commit` turned the commit off. The
person did not always want these commits, and the rules for them were hard to follow. A person
wants to decide when a change goes into git, and what goes into each commit.

## Decision

**No todoer command makes a git commit. Each command changes the files in the working tree of the
current checkout. The person commits them.**

- There is no `--no-commit` option.
- `pr submit` writes `.todoer/prs/<name>/` in the working tree of the current checkout. It prints
  the git command that commits the folder.
- `pr apply --merge` applies the patch with `git apply --3way`, removes the folder and the todos
  of the pull request, and stages all these changes with `git add`. It prints
  `git commit -m "todoer: merge <name>"`.
- `pr close` removes the folder and prints the git command that commits the removal.
- The todo commands only change the todo files.
- `pr apply --merge` and `pr apply` change the index only through `git apply` and `git add`.
- todoer never pushes or fetches.

## Consequences

- Other people see a todo, a pull request or a review only after the person commits and pushes.
- A merge commit has the person who commits as its author, not the author of the pull request.
- Other staged changes go into the merge commit. Stage nothing else before a merge.
- A failed command leaves no partial commit. A merge that does not apply changes nothing.
