# A pull request carries its todo changes, and todos have no labels and no dependencies

## Context

The patch of a pull request did not contain `.todoer/`. A merge removed the todos that `--todo`
named. This failed for a pull request into a feature branch:

1. `feat-a` starts from `main`. `sub-1` starts from `feat-a`.
2. On `feat-a`, a merge of a pull request from `sub-1` removes the todo `sub-task`.
3. On `main`, the pull request from `feat-a` has no todos, so `sub-task` stays on `main`.

The todos that `feat-a` adds or changes also never got to `main`.

The patch could not carry the todos, because of labels and dependencies. A dependency is a link
between two todo files. The rules of the todo commands keep these links correct: no missing name in
`dependsOn`, no cycle, and no label in `dependsOnLabels` that no other todo has. A patch is only
text, and it does not run these rules. The fixes in a patch come from the todos of the work branch.
At the merge, the target branch can have other todos.

## Decision

**Todos have no labels and no dependencies. The patch of a pull request carries all the changes of
the work, also in `.todoer/`, except the folder of the pull request itself.**

- A todo file has no front matter.
- There are no `labels` and `edit` commands, no label rules and no `check` command. `ls` has no
  `--label`.
- Every git command of a submit excludes only `.todoer/prs/<name>/`. A new todo, a changed todo, a
  removed todo, `.todoer/areas` and the folders of other pull requests go into the patch.
- The work removes its todos with `todoer done`, and commits the removal. The todos of a pull
  request are the todo files that `pr.patch` removes. `pr.md` does not list them. There is no
  `--todo` option.
- `pr apply` and `pr apply --merge` give `--exclude` to `git apply` for each todo file that the
  patch removes. Then they remove these todos by name, and stage the removal. Thus an edit of the
  todo on the current branch does not stop the patch.

## Consequences

- A todo that a merge into `feat-a` removes is also removed on `main`, when the pull request from
  `feat-a` merges into `main`.
- A todo that the work adds gets to the branch that the pull request goes into.
- An open pull request on `feat-a` gets to `main` with the pull request from `feat-a`. Merge or
  close the pull requests of a feature branch before its submit, unless they must move with it.
- A change of a todo file other than a removal can conflict, as any other file.
- The review shows the changes in `.todoer/`. A reviewer can comment on them.
- todoer cannot tell that a todo waits for another todo. Write the order of the work in the text
  of the todo.
- The front matter parser reads no lists. Only `pr.md` has front matter.
