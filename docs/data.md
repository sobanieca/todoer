# todoer - Files

todoer writes plain files. A person and an agent can read all of them. Do not edit them by hand
while `todoer ui` runs. Use the commands or the page.

```
todoer.json                    the settings (optional)
.todoer/
  .gitignore                   ignores worktrees/ and runs/
  todos/<id>-<slug>.md         one todo: front matter and a markdown description
  assets/<id>/<n>-<name>.png   the images of a todo
  prs/<id>/
    pr.md                      the pull request: front matter and the description
    v1.patch, v2.patch, ...    git format-patch series, for git am
    v1.diff, v2.diff, ...      the combined diff of each revision, for the review
    review.v1.md, ...          the review of each revision
  runs/<time>.log              the output of agents that the page started
  worktrees/<ids>/             the worktrees of todoer start (not under Claude Code)
```

You can commit `.todoer/` to share the todos through git, or add it to `.gitignore` to keep them on
your machine. A patch never contains `.todoer/`.

## A todo

```markdown
---
id: 7f3a
title: Login crashes on an empty password
status: in-progress
groups: ['tests refactoring', 'bump some-npm-package version to 0.3.5']
dependsOn: ['todo:ab12', 'pr:k2m9']
created: 2026-10-06T08:00:00.000Z
updated: 2026-10-06T09:12:00.000Z
branch: todoer/7f3a
worktree: .todoer/worktrees/7f3a
pr: k2m9
---

Steps to reproduce...

![1-crash.png](../assets/7f3a/1-crash.png)
```

A list field is a JSON array. `dependsOn` holds `todo:<id>` and `pr:<id>` references. `pr.md` has a
`dependsOn` field of the same form.

## A review

```markdown
---
revision: 1
verdict: changes-requested
updated: 2026-10-06T10:00:00.000Z
---

Close. Three small things.

### c-8k2f `src/api/items.js:42`

> const items = await db.query(sql) Catch the error here.

### c-9x1a `src/old-name.js`

Rename the file to items.js.

### c-2b3c `src/legacy/`

Move this folder under lib/.
```

`verdict` is `pending`, `changes-requested` or `approved`. A line comment keeps the line that it
names, after `>`.
