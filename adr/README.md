# Architecture decisions

Each file is one accepted decision. Presence means accepted. There are no numbers and no status
field. When a decision changes, replace the file. When it is dropped, delete the file.

## Data

| Decision                                                                    | Summary                                                       |
| --------------------------------------------------------------------------- | ------------------------------------------------------------- |
| [a-todo-is-a-markdown-file](a-todo-is-a-markdown-file.md)                   | a todo is a markdown file with a status field; its path is its id |
| [patch-based-pull-requests](patch-based-pull-requests.md)                   | a pull request is a folder of patches on the main branch      |
| [review-comment-targets](review-comment-targets.md)                         | a comment names lines, a file, a folder or the pull request   |
| [the-review-is-for-the-latest-patch](the-review-is-for-the-latest-patch.md) | a new submit replaces the patch and removes the review        |
| [dependencies-between-todos](dependencies-between-todos.md)                 | todos depend on todos, with no cycle                          |
| [labels-replace-groups](labels-replace-groups.md)                           | a label is a name in the `labels` list of a todo              |
| [code-owners](code-owners.md)                                               | teams and code owners filter the files of a review            |

## Workflow

| Decision                                                                        | Summary                                                  |
| ------------------------------------------------------------------------------- | -------------------------------------------------------- |
| [todoer-commits-only-on-the-main-branch](todoer-commits-only-on-the-main-branch.md) | pull requests and reviews are commits on the main branch; todo commands do not commit |
| [agents-make-their-own-worktrees](agents-make-their-own-worktrees.md)           | todoer does not make work worktrees                      |
| [agent-skill-prints-the-workflow](agent-skill-prints-the-workflow.md)           | `todoer skill` is the one source of the agent workflow   |
| [review-with-an-agent](review-with-an-agent.md)                                 | a person reviews the patch by talking with an agent      |

## Runtime

| Decision                                                                    | Summary                                           |
| --------------------------------------------------------------------------- | ------------------------------------------------- |
| [rust-binary-with-few-dependencies](rust-binary-with-few-dependencies.md)   | one Rust binary; the only crate is `ureq`         |
