---
name: todoer
description: Claim todoer todos, implement them in a git worktree and submit the work as a todoer pull request, review a todoer pull request with the person, or address its review. Use when asked to work on todoer todos (markdown files with a status field, like todo/fix-login.md) or on todoer pull requests in .todoer/prs/.
---

# todoer

todoer keeps the todos and the pull requests of a git project as plain files.

- A todo is a markdown file with a `status` field in its front matter: `todo`, `in-progress`, `done`
  or `dropped`. Its path from the repository root is its id.
- A pull request is a folder `.todoer/prs/<slug>/` on the main branch. It holds the patch of the
  latest submit and its review.
- todoer commits only on the main branch, in the main checkout. Run todoer from the main checkout
  or from any worktree of the project.

## Implement a todo

1. Read the todo file. `todoer ls --ready` lists the todos that can start now.

2. Claim the todo before you start:

       todoer claim <path>

   The claim sets `status: in-progress` and `assignee: <your git email>`, and commits this on the
   main branch. If another person has the todo, or the todo waits for other todos, the command
   stops. Then stop and tell the person.

3. Make a worktree from the main branch, on a new branch:

       git worktree add -b <branch> <folder> <main branch>

   If your tool makes worktrees for you, use that worktree.

4. Work only in the worktree. Never change the main checkout. Run the checks of the project, then
   commit your work on the branch.

5. Submit the pull request from the worktree:

       todoer pr submit -t "<short title>" -m "<what you changed and why>" --todo <path>

   Give `--todo` once for each todo. The command prints the slug of the pull request. Do not push
   the branch. Do not open a pull request on GitHub or on another host.

6. Tell the person the slug. They review it with you, or with `todoer pr review <slug>`.

## Address a review

1. Read the review:

       todoer pr summary <slug>
       todoer pr details <slug> [file...]

   A comment has an id like `c1` and a target:

   - `path:42` - line 42 of the new version. `path:40-45` - lines 40 to 45.
   - `path:old:12` - line 12 of the old version, a removed line.
   - `path` - the whole file, for example "rename it". `folder/` - the whole folder.
   - no target - the whole pull request.

2. Go to the worktree of the branch. Do the changes that the comments ask for. A comment can name
   a file that the pull request does not change yet: change that file too. For a rename or a move,
   use `git mv`. If a comment is not clear, ask the person.

3. Commit, then submit again from the worktree:

       todoer pr submit --update -m "<what you changed for each comment>"

   `--update` replaces the patch and removes the review. Do every comment first: todoer does not
   check them. If a reviewer still writes the review, the command stops. Then wait.

4. If you do not do a comment, say why in `-m`. The reviewer reads it in the `## Update` section.

## Review a pull request with the person

The person reviews a pull request by talking with you. Read the patch, explain it, and add the
comments that the person asks for.

    todoer pr summary <slug>
    todoer pr details <slug> [file...] [--context 5] [--lines 40-80]
    todoer pr comment <slug> <target> "<text>"
    todoer pr approve <slug>
    todoer pr request-changes <slug> -m "<what must change>"

- `pr details` shows the line numbers of the new version. Use them in the target, for example
  `src/app.rs:42`.
- Write the comment text as the person says it. If the line is not clear, ask the person.
- A new comment is a draft. Send the review with `approve` or `request-changes` only when the
  person asks. These commands commit the review.

## Plan work for the person

    todoer add -t "<title>" -p <folder> -l <label> -d <path it depends on> "<what and how to check it>"
    todoer dep add <path> <path it depends on>
    todoer label add <path> <label>
    todoer progress <label>

Add a dependency only when a todo cannot start before the other todo is done.

## Rules

- Do not run `todoer pr merge` or `todoer pr close` unless the person asks.
- Do not edit the files in `.todoer/` yourself. Use the commands.
- Do not use `--force` unless the person asks.
- Add `--json` to `todoer ls`, `todoer labels`, `todoer progress` and `todoer pr ls` for output
  that you parse.
