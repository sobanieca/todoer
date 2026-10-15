# todoer - Files

todoer writes plain files. A person and an agent can read all of them. Commit them as other files.

```
todoer.json                     the settings of the project (optional)
<any folder>/<slug>.md          a todo: front matter with a status field, and a description
.todoer/
  prs/<slug>/
    pr.md                       title, author, branch, base, todos, the description, the update
    pr.patch                    git format-patch series of the latest submit, for git am
    pr.diff                     the combined diff of the latest submit, for the review
    review.md                   the review of the latest patch: the verdict and the comments
  teams/<team>.md               the members of a team, by git email
  codeowners                    the code owners, in the CODEOWNERS syntax
```

A patch never contains `.todoer/`. `merge` and `close` remove the folder of the pull request. git
history keeps it.

## A todo

```markdown
---
title: Port the CLI parser
status: in-progress
assignee: sobanieca@gmail.com
labels: [rust-port, cli]
dependsOn: [todo/rust/port-config.md]
---

The description. Images are plain markdown links to files near the todo.
```

- `status` is `todo`, `in-progress`, `done` or `dropped`.
- `assignee` is a git email. `todoer claim` sets it.
- A list is `[a, b]`. Put an item in double quotes when it has a comma.
- todoer changes only the fields that a command names. The other lines stay as they are.

## A pull request - `pr.md`

```markdown
---
title: Port the CLI parser
author: agent-1@example.com
branch: port-cli
base: 3f2a91c0d8e6b3c7a1d2e4f5a6b7c8d9e0f1a2b3
todos: [todo/rust/port-cli.md]
---

What changed and why.

## Update

What changed for the last review.
```

- The first `-m` is the description. The `-m` of each later submit replaces `## Update`.

## A review - `review.md`

```markdown
---
verdict: changes-requested
reviewer: sobanieca@gmail.com
---

Close. Three small things.

### c1 `src/cli/args.rs:42` sobanieca@gmail.com

>     let b = 2;

Use the constant.

### c2 `src/old.rs` sobanieca@gmail.com

Rename the file to lib.rs.

### c3 pr sobanieca@gmail.com

Add a test.
```

- The review is for the latest patch. `todoer pr submit --update` removes the file. git history
  keeps it.
- `verdict` is `pending`, `changes-requested` or `approved`.
- A comment header has the id, the target in backticks (or `pr`) and the author.
- A line comment keeps the text of its lines, after `>`.
- A comment that is not committed is a draft.
