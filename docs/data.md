# todoer - Files

todoer writes plain files. A person and an agent can read all of them. Commit them as other files.

```
.todoer/
  todos/<name>.md               a todo: optional front matter and a description
  prs/<name>/
    pr.md                       author, from, into, base, todos, the description
    pr.patch                    the latest submit as one squashed commit, for git am
    pr.diff                     the combined diff of the latest submit, for the review
    review.md                   the review of the latest patch: the comments
  labels/<label>.md             a label rule: which todos must have the label, in plain words
  areas                         the areas of the code, with the patterns of their paths
```

A patch never contains `.todoer/`. `pr submit` writes the folder of a pull request on the branch
that it goes into. `merge` and `close` remove the folder. git history keeps it.

## A todo - `.todoer/todos/<name>.md`

```markdown
---
assignee: you@example.com
labels: [auth]
dependsOn: [fix-the-login-crash]
dependsOnLabels: [auth-backend-refactor]
---

What to do and how to check it. Images are plain markdown links.
```

- The name matches `^[a-z0-9-]+$`. It is the id and the title. The folder has no subfolders: use
  labels to collect todos.
- The front matter is optional. A file with no front matter is a new todo.
- `assignee` is a git email. `todoer claim` sets it and commits the file. A todo with an assignee
  is in progress.
- `dependsOn` lists names. A todo waits while a todo of the list exists.
- `dependsOnLabels` lists labels. A todo waits while another todo has a label of the list. When
  no other todo has the label, `done`, `pr merge`, `label rm` and `labels rm` remove it from the
  list.
- The file exists only while the work is open. `todoer done` and `todoer pr merge` remove it.
- todoer reads only the `*.md` files in `.todoer/todos/`.
- A list is `[a, b]`. Put an item in double quotes when it has a comma.
- todoer changes only the fields that a command names. The other lines stay as they are.

## A label rule - `.todoer/labels/<label>.md`

```markdown
Add this label to a todo that changes the login, the sessions or the tokens.
Do not add it to a todo that only changes the docs of the login.
```

- The file name is the label. It matches `^[a-z0-9-]+$`. The body is the rule.
- todoer does not read the rule. The agent reads it after `todoer add`, and adds the label to each
  todo that the rule matches.
- `todoer labels rule` writes the file. `todoer labels rm` and `todoer labels mv` remove and rename
  it. They do not commit. Commit the rules as other files.
- `todoer check` reports a file with a bad name, and an empty rule.

## A pull request - `pr.md`

```markdown
---
author: you@example.com
from: fix-login
into: main
base: 3f2a91c0d8e6b3c7a1d2e4f5a6b7c8d9e0f1a2b3
todos: [fix-the-login-crash]
---

What changed and why.

## Update

What changed for the last review.
```

- The name of the folder matches `^[a-z0-9-]+$`. It is the id and the title of the pull request.

- `from` is the branch of the work. `into` is the branch that the pull request goes into, from
  `--branch`. `base` is the merge base of the latest submit.
- The first `-m` is the description. The `-m` of each later submit replaces `## Update`.

## A review - `review.md`

```markdown
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
- `todoer pr comment` writes the file. It does not commit it. Commit it when the review is done.
- There is no verdict. A merge accepts the pull request.
- A comment header has the id, the target in backticks (or `pr`) and the author.
- A line comment keeps the text of its lines, after `>`.
