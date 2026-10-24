# todoer - Files

todoer writes plain files. A person and an agent can read all of them. todoer never commits them.
Commit them as other files.

```
.todoer/
  todos/<name>.md               a todo: plain markdown, what to do
  prs/<name>/
    pr.md                       author, base, the description
    pr.patch                    the latest submit as one squashed commit, for git am
    review.md                   the review of the latest patch: the comments
  areas                         the areas of the code, with the patterns of their paths
```

A patch contains all the changes of the work, also in `.todoer/`, except the folder of its own
pull request. `pr submit` writes the folder of a pull request in the working tree of the branch
that it goes into. `merge` and `close` remove the folder. git history keeps it after you commit it.

## A todo - `.todoer/todos/<name>.md`

```markdown
What to do and how to check it. Images are plain markdown links.
```

- The name matches `^[a-z0-9-]+$`. It is the id and the title. The folder has no subfolders.
- The file is plain markdown. todoer reads no fields from it. A todo has no assignee and no state.
- The file exists only while the work is open. `todoer done` and `todoer pr apply --merge` remove
  it.
- todoer reads only the `*.md` files in `.todoer/todos/`.

## A pull request - `pr.md`

```markdown
---
author: you@example.com
base: 3f2a91c0d8e6b3c7a1d2e4f5a6b7c8d9e0f1a2b3
---

What changed and why.
```

- The name of the folder matches `^[a-z0-9-]+$`. It is the id and the title of the pull request.

- `base` is the merge base of the latest submit. `pr.md` does not name a branch. The pull request
  goes into the branch that holds the folder. It does not need the branch of the work.
- The body is the description, from `-m`. `--update` writes the whole file again. It copies the
  description of the old file when `-m` is not given.
- `pr.md` does not list the todos. The todos of the pull request are the todo files that
  `pr.patch` removes.

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
