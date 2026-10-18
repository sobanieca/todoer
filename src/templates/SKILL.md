---
name: todoer
description: Add todoer todos with the labels of the label rules, claim todoer todos, implement them in a git worktree and submit the work as a todoer pull request, review a todoer pull request with the person, or address its review. Use when asked to add a todo, an issue, a task or a bug in a project with a .todoer/ folder, to work on todoer todos (files in .todoer/todos/, like .todoer/todos/fix-login.md) or on todoer pull requests in .todoer/prs/.
---

# todoer

todoer keeps the todos and the pull requests of a git project as plain files.

- A todo is a file `.todoer/todos/<name>.md`. The name is its id and its title. The front matter
  has `assignee`, `labels`, `dependsOn` and `dependsOnLabels`. All are optional. A todo with an
  assignee is in progress.
- A pull request is a folder `.todoer/prs/<name>/` on the branch that it goes into. The name is
  its id and its title. It holds the patch of the latest submit and its review.
- todoer reads and writes the files of the current checkout, on any branch. `claim`, `unclaim` and
  the pull request commands commit only their own files. The other todo commands do not commit.

## The workflow

1. The person adds a todo with `todoer add`.
2. You claim the todo. The claim is a commit on the current branch of the person.
3. You implement the todo in a worktree, on a new branch from the branch of the person.
4. You submit a pull request into the branch of the person. The submit commits it there.
5. The person reviews the pull request, alone or with you, and commits the review.
6. An agent does the comments and updates the pull request. This removes the review.
7. The person merges the pull request with `todoer pr merge`. There is no approve step.

## Claim todos

The person can ask for one todo, a list of todos or a label.

    todoer claim <name>...
    todoer claim --label <label>

- The claim sets `assignee: <your git email>` and commits the todo files on the current branch.
- If the checkout has other changes that are not committed, the claim does not commit. Then tell
  the person to commit the todo files.
- If another person has a todo, or a todo waits for a todo that you do not claim, the command
  stops. Then stop and tell the person.
- Do not push. The person pushes when they want.

## Implement todos

1. Read each todo with `todoer show <name>`. Note the current branch of the person:
   `git branch --show-current`. This is the branch that the pull request goes into, unless the
   person names another branch.

2. Make a branch for your work from that branch, in a worktree:

       git worktree add -b <branch> <folder> <branch of the person>

   If your tool makes worktrees for you, use that worktree. Make sure that it starts from the
   branch of the person.

3. Work only in your worktree. Implement the todos in dependency order. Run the checks of the
   project, then commit your work.

4. Submit the pull request from your worktree:

       todoer pr submit --branch <branch of the person> -n <pr name> -m "<what you changed and why>" --todo <name>

   - The pull request name has lowercase letters, digits and hyphens, and starts and ends with a
     letter or digit, for example `fix-login-crash`.
   - Give `--todo` once for each todo that the pull request does.
   - The patch has all the commits of your branch after the merge base with `--branch`, squashed
     into one commit.
   - The command writes `.todoer/prs/<pr name>/` on the branch `--branch` and commits it there.
     If the checkout of that branch has other changes, todoer does not commit. Then tell the
     person to commit the folder.
   - Do not push. Do not open a pull request on GitHub or on another host.
   - The pull request does not need your branch. The person can remove your worktree and your
     branch after the submit.

5. Tell the person the pull request name and the branch that it goes into.

## Review a pull request with the person

Review in the checkout of the branch that the pull request goes into. Read the patch, explain it,
and add the comments that the person asks for.

    todoer pr ls
    todoer pr summary <pr>
    todoer pr details <pr> [file...] [--context 5] [--lines 40-80] [-a area,...] [-x area,...]
    todoer pr comment <pr> <target> "<text>"

- `-a security,tests` shows only the files in these areas of `.todoer/areas`. `-x unit-tests`
  hides the files in these areas. Use them when the person asks to review a part of the code.
- `pr details` shows the line numbers of the new version. Use them in the target, for example
  `src/app.rs:42`.
- A target is `path:42`, `path:40-45`, `path:old:12` (a removed line), `path` (a file),
  `folder/`, or no target for the whole pull request.
- Write the comment text as the person says it. If the line is not clear, ask the person.
- `pr comment` writes `.todoer/prs/<pr>/review.md` and does not commit it. When the person says
  that the review is done, and asks you to commit it:

      git add .todoer/prs/<pr>/review.md && git commit -m "Review <pr>" -- .todoer/prs/<pr>/review.md

- There is no approve and no request for changes. To accept, the person merges. To ask for
  changes, the person gives the committed review to an agent.

## Address a review

1. Read the review in the checkout of the branch that the pull request goes into:

       todoer pr review <pr>

   The branch of this checkout is the branch that the pull request goes into (`<into>`). `pr.md`
   names the commit of the patch in `base`. Note the path of this checkout.

2. Make a new worktree and a new branch for the fix, with your tool or with git. Start it at
   `base`:

       git worktree add -b <branch> <folder> <base>

   If your tool starts the worktree at another commit, run `git reset --hard <base>` in the new
   worktree before you change a file.

3. Restore the commit of the pull request in your worktree:

       git am <checkout of into>/.todoer/prs/<pr>/pr.patch

   The patch always applies on `base`. Do not apply it on the latest `<into>`: it can conflict. The
   branch of the first work is not necessary. The person can remove it after a submit.

4. Do the changes that the comments ask for. A comment can name a file that the pull request does
   not change yet: change that file too. For a rename or a move, use `git mv`. If a comment is not
   clear, ask the person.

5. Commit, then update the pull request:

       todoer pr submit --branch <into> -n <pr> --update -m "<what the pull request changes and why>" --todo <name>

   `-n <pr>` finds the pull request, so the name of your branch is not important. `--update`
   writes the pull request again and removes the review. `-m` replaces the description and
   `--todo` replaces the todos. Without them, the update keeps the old ones and says so. Do every
   comment first: todoer does not
   check them. If you do not do a comment, tell the person.

## Plan work for the person

When the person asks you to add a todo, an issue, a task or a bug, add a todo with `todoer add`.
Apply the label rules (see below).

    todoer add <name> -l <label> -d <name it depends on> "<what and how to check it>"
    todoer add <name> -D <label it depends on> "<what and how to check it>"
    todoer dep add <name> <name it depends on>
    todoer dep add <name> -l <label it depends on>
    todoer label add <name> <label>
    todoer labels rm <label>
    todoer labels mv <old> <new>
    todoer labels rule <label> "<which todos must have the label>"
    todoer progress <label>

A name has lowercase letters, digits and hyphens, and starts and ends with a letter or digit, for
example `fix-login-crash`. Add a dependency only when a todo cannot start before the other todo is
done. When a todo waits for all the todos of a label, depend on the label: then a new todo with the
label also blocks the todo.

A label rule tells in plain words which todos must have a label. todoer does not read the rules.
You apply them:

1. Before `todoer add`, run `todoer labels`. It prints the rule of each label below the label.
2. Read each rule, and compare it with the name and the text of the new todo.
3. Give `-l <label>` to `todoer add` for each rule that the todo matches. For a todo that exists,
   run `todoer label add <name> <label>`.
4. Tell the person which labels you added and why. If a rule is not clear, ask the person.

Also follow the rules when you change the text of a todo. Do not remove a label that the person
added.

## Rules

- Do not run `todoer pr merge` or `todoer pr close` unless the person asks.
- Do not edit the files in `.todoer/` yourself. Use the commands.
- Do not use `--force` or `--no-commit` unless the person asks.
- Do not push. todoer never pushes.
- Add `--json` to `todoer ls`, `todoer show`, `todoer labels`, `todoer progress` and
  `todoer pr ls` for output that you parse.
