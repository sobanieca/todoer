# Architecture decisions

Each file is one accepted decision. Presence means accepted. There are no numbers and no status
field. When a decision changes, replace the file. When it is dropped, delete the file.

## Data

| Decision                                                                | Summary                                                      |
| ----------------------------------------------------------------------- | ------------------------------------------------------------ |
| [plain-files-in-the-main-checkout](plain-files-in-the-main-checkout.md) | todos, pull requests and reviews are files in `.todoer/`     |
| [patch-based-pull-requests](patch-based-pull-requests.md)               | a pull request is a patch series plus a combined diff        |
| [review-comment-targets](review-comment-targets.md)                     | a comment names a line, a file or a folder                   |
| [groups-are-names-on-todos](groups-are-names-on-todos.md)               | a group is a name in the `groups` field of its todos         |
| [dependencies-between-items](dependencies-between-items.md)             | todos and pull requests depend on each other, with no cycle  |
| [image-size-limit](image-size-limit.md)                                 | todoer refuses an image above `maxImageKb`, it never resizes |

## Workflow

| Decision                                                              | Summary                                                      |
| --------------------------------------------------------------------- | ------------------------------------------------------------ |
| [one-worktree-per-request](one-worktree-per-request.md)               | `todoer start` makes one branch and one worktree per request |
| [agent-skill-prints-the-workflow](agent-skill-prints-the-workflow.md) | `todoer skill` is the one source of the agent workflow       |

## Runtime

| Decision                                                      | Summary                                                    |
| ------------------------------------------------------------- | ---------------------------------------------------------- |
| [dependency-free-deno-binary](dependency-free-deno-binary.md) | plain JavaScript on Deno, no imports from outside the repo |
| [lazy-dependency-tree](lazy-dependency-tree.md)               | the page opens a dependency tree one level at a time       |
| [local-page](local-page.md)                                   | an imp page on localhost, with host and origin checks      |
