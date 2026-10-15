---
title: Teams, code owners and the owner filter
status: done
labels: [cli-reshape]
dependsOn: [todo/pr-review-cli.md]
---

Show the files of a pull request that a person or a team owns.

## Files

- `.todoer/teams/<team>.md` lists the members of a team by git email. A member can be an agent.
- `.todoer/codeowners` uses the CODEOWNERS syntax. When it is missing, todoer reads
  `.github/CODEOWNERS`.
- The last rule that matches a path wins. The patterns use the gitignore rules.
- "Me" is `git config user.email` and each team that has this email.

## Filters

- `--owner <team|person>`, `--mine` and `--unowned`.
- They apply to `pr summary`, to `pr details` with no file, and to `pr review`.
- `todoer pr ls --mine` lists the pull requests that change files that I own.
- For a renamed file, the owners of the old and the new path count.

Write the gitignore pattern matcher in the crate, with many tests.

Approval rules before a merge are not in this item. Add a new todo for them when necessary.
