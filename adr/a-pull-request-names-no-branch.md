# A pull request names no branch, and todoer guesses no name

## Context

`pr.md` held `from`, the branch of the work, and `into`, the branch that the pull request goes into.
`into` repeated the branch that holds the folder. After a git merge of that branch into another
branch, `into` was wrong. `from` let a submit without `-n` find the pull request of its branch. But
the branch of the work can be gone, and another person updates from a branch with another name.
Without `-n`, todoer made the name from the subject of the first commit. A rebase or a new branch
changed that name, and a `-2` suffix then made a second pull request with no warning. A pull
request must be self-contained, and the person must always know which pull request a command
changes.

## Decision

**`pr.md` names no branch. `--branch` and `-n` are necessary on each submit. todoer does not make
a name of a pull request and does not find a branch for the person.**

- The branch that holds `.todoer/prs/<name>/` is the branch that the pull request goes into.
- `pr.md` holds only the author, the base and the todos.
- `pr submit` finds the pull request by `-n`, only in the open pull requests of `--branch`.
- A submit without `--update` never changes an open pull request. It stops and names
  `-n <pr> --update`.
- `--update` with a name that `--branch` does not have stops. The error lists the open pull
  requests of `--branch`.
- Each error names `--branch` and the pull request, so the person can copy the next command.
- `pr apply` prints the update command with a placeholder for `--branch`. It does not look for the
  branch in other local branches.

## Consequences

- Each update needs `-n <pr> --update`. The name of the branch of the update is not important.
- `pr ls` and `pr view` show no branch.
- When git merges a branch with an open pull request into another branch, the pull request goes
  into that other branch too.
