---
title: Pull requests as patch files on the main branch
status: todo
labels: [cli-reshape]
dependsOn: [todo/todos-as-files.md]
---

A pull request is a folder on the main branch. People and agents share it through git.

```
.todoer/prs/<slug>/
  pr.md          title, author, base, status, todos
  v1.patch       git format-patch series, for git am
  v1.diff        the combined diff of the revision, for the review
  review.v1.md   the review of the revision
```

- A command takes the path of the folder, or only the slug.
- In a review worktree, the pull request argument is optional.

## Commands

| Command                              | Does                                                     |
| ------------------------------------ | -------------------------------------------------------- |
| `todoer submit [-t <title>]`         | sends the commits of the branch as the next revision     |
| `todoer prs [--all]`                 | lists the open pull requests                             |
| `todoer checkout <pr> [--rev N]`     | makes a worktree with the patch applied, to run the code |
| `todoer pr merge <pr>`               | applies the patch on the main branch                     |
| `todoer pr close <pr>`               | closes a pull request without a merge                    |

- `submit` runs in a worktree, on a branch that is not the main branch. Agents make their own
  worktrees, so remove `todoer start`.
- `submit` writes into the main checkout and commits only the folder:
  `git commit -- .todoer/prs/<slug>`. Other staged changes stay as they are.
- `checkout` makes the branch `review/<slug>` from `base` and runs `git am --3way`. It opens
  `$SHELL` in the worktree, and `exit` goes back. With `--print`, or with no terminal, it prints
  the path.
- `merge` runs `git am --3way`. It then removes the folder in one more commit. git history keeps
  the folder.

## ADRs

Replace `patch-based-pull-requests` and `one-worktree-per-request`.
