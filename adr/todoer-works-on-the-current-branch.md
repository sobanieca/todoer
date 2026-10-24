# todoer works on the current branch and knows no main branch

## Context

todoer committed every pull request, every merge and every verdict on the main branch, in the main
checkout. The person had to keep the main branch checked out there, and `todoer.json` had to name
the main branch. A person who works on a feature branch, with agents on branches of that feature
branch, could not use todoer there.

## Decision

**todoer reads and writes the `.todoer/` folder of the current checkout, on any branch. It has no
main branch, no main checkout and no settings file.**

- No command commits (see [todoer-never-commits](todoer-never-commits.md)).
- `pr submit` writes the pull request into the branch that is checked out. `--branch` names the
  branch of the work.
- The git commands read the diff and the patch of a submit, and the old file for a review.
  `pr apply` also applies the patch.
- A lock file `todoer.lock` in the git common folder lets one todoer command write at a time.
- Outside a git repository, only `todoer ls` reads many projects. It finds each git project with
  `.todoer/todos/` in the current folder and in its subfolders.
- `todoer ls` reads the branch that is checked out in each project. It shows the path of the project
  relative to the current folder, and the branch, above the todos of the project.
- `todoer ls` does not look into a project, a hidden folder or a linked worktree.
- All other commands work on one project. Outside a git repository, they stop with an error.

## Consequences

- A pull request can go into any branch, also a feature branch. The pull requests of a feature
  branch stay on that branch.
- To submit into a branch, the person checks out that branch first.
- A person sees the todos of all projects in one list. To change a todo or a pull request, the
  person goes to the folder of the project. todoer never guesses the project of a change.
- `todoer ls` does not show the todos of a linked worktree. Run `todoer ls` in the worktree.
