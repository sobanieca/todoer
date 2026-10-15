# Code owners filter the files of a review

## Context

A large pull request touches the files of many teams. A reviewer wants to see the files that they
own. Agents can also own code.

## Decision

**Teams are files `.todoer/teams/<team>.md` that list git emails. The rules are in
`.todoer/codeowners`, else in `.github/CODEOWNERS`.**

- The patterns use the gitignore rules. The last rule that matches a path wins.
- "Me" is `git config user.email` and each team that has this email.
- `--owner`, `--mine` and `--unowned` filter `pr summary`, `pr details` and `pr review`.
- For a renamed file, the owners of the old and the new path count.

## Consequences

- Approval rules before a merge are not part of this decision.
