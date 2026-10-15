---
title: Pull requests as patch files on the main branch
status: done
labels: [cli-reshape]
dependsOn: [todo/migrate-to-rust.md]
---

A pull request is a folder on the main branch. People and agents share it through git.

```
.todoer/prs/<slug>/
  pr.md          title, author, branch, base, revision, todos
  v1.patch       git format-patch series, for git am
  v1.diff        the combined diff of the revision, for the review
  review.v1.md   the review of the revision
```

- A command takes the path of the folder, or only the slug.
- In a review worktree, the pull request argument is optional.
- We will probably remove revisions: a pull request keeps only its latest patch. Decide this
  before `todo/compare-revisions.md`. Until then, keep `v<N>` files.

## Commands

Every pull request command is under `todoer pr`.

| Command                                       | Does                                                     |
| --------------------------------------------- | -------------------------------------------------------- |
| `todoer pr submit [-t <title>] [--todo path]` | sends the commits of the branch as the next revision     |
| `todoer pr ls [--json]`                       | lists the open pull requests                             |
| `todoer pr checkout <pr> [--rev N] [--print]` | makes a worktree with the patch applied, to run the code |
| `todoer pr merge <pr>`                        | applies the patch on the main branch                     |
| `todoer pr close <pr>`                        | closes a pull request without a merge                    |

- `submit` runs in a worktree, on a branch that is not the main branch. Agents make their own
  worktrees, so remove `todoer start`.
- `--todo` names a todo of the pull request. Give it once for each todo. A next revision keeps the
  todos of the earlier revisions.
- Every commit of todoer goes on the main branch, in the main checkout. Before `submit`, `merge`,
  `close` and a verdict, todoer checks that the main checkout is on the main branch. If it is on a
  feature branch, todoer stops and tells the person to check out the main branch there.
- `submit` commits only its own files: `git commit -- <files>`. Other staged changes stay as they
  are. A draft review in the same folder stays a draft.
- `checkout` makes the branch `review/<slug>` from `base` and runs `git am --3way`. It opens
  `$SHELL` in the worktree, and `exit` goes back. With `--print`, or with no terminal, it prints
  the path.
- `merge` runs `git am --3way`. It then sets `status: done` on the todos of the pull request and
  removes the folder, in one more commit. git history keeps the folder. It also removes the review
  worktree and the branch `review/<slug>`.
- `close` removes the folder in one commit. The todos keep their status. Use `todoer unclaim` to
  free them.

## ADRs

Replace `patch-based-pull-requests` and `one-worktree-per-request`.
