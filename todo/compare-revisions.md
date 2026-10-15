---
title: Compare two revisions of a pull request
status: todo
labels: [cli-reshape]
dependsOn: [todo/pr-review-cli.md]
---

`todoer pr details <pr> --since N` shows the changes from revision `N` to the latest revision. Use
`git range-diff`, or apply both revisions on two branches and compare them.

When a new revision comes, move each open comment to the same line text in the new revision. Mark
the other comments as outdated. `pr summary` shows the number of outdated comments.
