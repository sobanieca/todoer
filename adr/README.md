# Architecture decisions

Each file is one accepted decision. Presence means accepted. There are no numbers and no status
field. When a decision changes, replace the file. When it is dropped, delete the file.

## Data

| Decision                                                                    | Summary                                                       |
| --------------------------------------------------------------------------- | ------------------------------------------------------------- |
| [a-todo-is-a-file-in-the-todoer-folder](a-todo-is-a-file-in-the-todoer-folder.md) | a todo is `.todoer/todos/<name>.md`; the name is its id; no assignee |
| [patch-based-pull-requests](patch-based-pull-requests.md)                   | a pull request is a folder with a patch; submit it in the checkout of the branch that it goes into |
| [review-comment-targets](review-comment-targets.md)                         | a comment names lines, a file, a folder or the pull request   |
| [a-pull-request-names-no-branch](a-pull-request-names-no-branch.md)         | `pr.md` names no branch; `--branch` (the work) and `-n` are always given |
| [the-merge-message-is-the-pull-request-name](the-merge-message-is-the-pull-request-name.md) | the squash message is `todoer: merge <name>` |
| [an-update-replaces-the-pull-request](an-update-replaces-the-pull-request.md) | `-n <pr> --update` writes the pull request again and removes the review |
| [a-pull-request-carries-its-todo-changes](a-pull-request-carries-its-todo-changes.md) | the patch holds the changes of `.todoer/` except its own folder; todos have no labels and no dependencies |
| [code-areas](code-areas.md)                                                 | the summary counts the changes in each area; `-a` and `-x` filter the review |

## Workflow

| Decision                                                                        | Summary                                                  |
| ------------------------------------------------------------------------------- | -------------------------------------------------------- |
| [todoer-works-on-the-current-branch](todoer-works-on-the-current-branch.md)     | no main branch; todoer works on the checked-out branch   |
| [todoer-never-commits](todoer-never-commits.md)                                 | no command commits; the person commits the changes      |
| [agents-make-their-own-worktrees](agents-make-their-own-worktrees.md)           | todoer does not make work worktrees                      |
| [agent-skill-prints-the-workflow](agent-skill-prints-the-workflow.md)           | `todoer skill` is the one source of the agent workflow   |
| [review-with-an-agent](review-with-an-agent.md)                                 | a person reviews the patch by talking with an agent      |
| [apply-a-pull-request-to-the-working-tree](apply-a-pull-request-to-the-working-tree.md) | `pr apply` puts a pull request in the working tree; `--merge` merges and stages it |
| [few-commands](few-commands.md)                                                 | one command for each job; options select the part of the job |
| [a-merge-is-the-approval](a-merge-is-the-approval.md)                           | no verdict; a merge accepts, a committed review asks for changes |

## Runtime

| Decision                                                                    | Summary                                           |
| --------------------------------------------------------------------------- | ------------------------------------------------- |
| [rust-binary-with-few-dependencies](rust-binary-with-few-dependencies.md)   | one Rust binary; the only crate is `ureq`         |
