# Architecture decisions

Each file is one accepted decision. Presence means accepted. There are no numbers and no status
field. When a decision changes, replace the file. When it is dropped, delete the file.

## Data

| Decision                                                                    | Summary                                                       |
| --------------------------------------------------------------------------- | ------------------------------------------------------------- |
| [a-todo-is-a-file-in-the-todoer-folder](a-todo-is-a-file-in-the-todoer-folder.md) | a todo is `.todoer/todos/<name>.md`; the name is its id |
| [patch-based-pull-requests](patch-based-pull-requests.md)                   | a pull request is a folder with a patch, on the branch that it goes into |
| [review-comment-targets](review-comment-targets.md)                         | a comment names lines, a file, a folder or the pull request   |
| [a-pull-request-names-no-branch](a-pull-request-names-no-branch.md)         | `pr.md` names no branch; `--branch` and `-n` are always given |
| [the-merge-message-is-the-pull-request-name](the-merge-message-is-the-pull-request-name.md) | the squash message is `todoer: merge <name>` |
| [an-update-replaces-the-pull-request](an-update-replaces-the-pull-request.md) | `-n <pr> --update` writes the pull request again and removes the review |
| [dependencies-between-todos](dependencies-between-todos.md)                 | todos depend on todos, with no cycle                          |
| [labels-replace-groups](labels-replace-groups.md)                           | a label is a name in the `labels` list of a todo              |
| [label-rules](label-rules.md)                                               | todoer stores a rule for each label; the agent applies it     |
| [code-areas](code-areas.md)                                                 | the summary counts the changes in each area; `-a` and `-x` filter the review |

## Workflow

| Decision                                                                        | Summary                                                  |
| ------------------------------------------------------------------------------- | -------------------------------------------------------- |
| [todoer-works-on-the-current-branch](todoer-works-on-the-current-branch.md)     | no main branch; `claim`, `unclaim` and pull request commands commit their own files |
| [agents-make-their-own-worktrees](agents-make-their-own-worktrees.md)           | todoer does not make work worktrees                      |
| [agent-skill-prints-the-workflow](agent-skill-prints-the-workflow.md)           | `todoer skill` is the one source of the agent workflow   |
| [review-with-an-agent](review-with-an-agent.md)                                 | a person reviews the patch by talking with an agent      |
| [apply-a-pull-request-to-the-working-tree](apply-a-pull-request-to-the-working-tree.md) | `pr apply` puts a pull request in the working tree, no commit |
| [a-merge-is-the-approval](a-merge-is-the-approval.md)                           | no verdict; a merge accepts, a committed review asks for changes |

## Runtime

| Decision                                                                    | Summary                                           |
| --------------------------------------------------------------------------- | ------------------------------------------------- |
| [rust-binary-with-few-dependencies](rust-binary-with-few-dependencies.md)   | one Rust binary; the only crate is `ureq`         |
