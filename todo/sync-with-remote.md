---
title: Share pull requests through a remote
status: todo
labels: [cli-reshape]
dependsOn: [todo/patch-pull-requests.md]
---

`todoer pr submit`, the verdicts, `pr merge` and `pr close` commit on the local main branch. Two
people on two machines see these commits only at the next pull. Todo commands, also `claim`, do not
commit, so todoer does not push them.

- When the main branch has an upstream, a pull request command fetches it and checks the pull
  request on the upstream version too.
- After the commit, todoer pushes the main branch. When the push is rejected, todoer fetches,
  rebases the commit, checks the pull request again and pushes again.
- When the check fails after the rebase, todoer stops with the reason and removes the local commit.
- Add `--no-push`, and a `push` key in `todoer.json`.
