---
name: todoer
description: Add todoer todos, implement them in a git worktree and submit the work as a todoer pull request, review a todoer pull request with the person, or address its review. Use only when the person explicitly asks to use todoer: to add a todoer todo, to work on a todoer todo (a file in .todoer/todos/, like .todoer/todos/fix-login.md) with todoer, or to review a todoer pull request in .todoer/prs/ or address its review. Do not use for other changes, also in a project with a .todoer/ folder.
---

# todoer

todoer keeps the todos and the pull requests of a git project as plain files.

- A todo is a file `.todoer/todos/<name>.md`. The name is its id and its title. The file is
  plain markdown: what to do and how to check it.
- A pull request is a folder `.todoer/prs/<name>/` on the branch that it goes into. The name is
  its id and its title. It holds the patch of the latest submit and its review.
- The patch holds all the changes of your branch, also in `.todoer/`. Only the folder of the pull
  request itself is not in it. The todos that the patch removes are the todos of the pull request.
- todoer reads and writes the files of the current checkout, on any branch. todoer never commits.
  The person commits the changes of todoer.

## The workflow

1. The person adds a todo with `todoer add`.
2. You implement the todo in a worktree, on a new branch from the branch of the person. You run
   `todoer done` for the todo in the worktree, and commit it with the work.
3. You submit a pull request into the branch of the person. Run the submit in the checkout of the
   person. It writes the pull request in that working tree.
4. The person reviews the pull request, alone or with you, and commits the review.
5. An agent does the comments and updates the pull request. This removes the review.
6. The person merges the pull request with `todoer pr apply <pr> --merge`. There is no approve
   step.

## Implement todos

The person can ask for one todo or a list of todos.

1. Read each todo with `todoer show <name>`. Note the current branch of the person:
   `git branch --show-current`. This is the branch that the pull request goes into, unless the
   person names another branch. Note the path of the checkout of that branch:
   `git rev-parse --show-toplevel`.

2. Make a branch for your work from that branch, in a worktree:

       git worktree add -b <branch> <folder> <branch of the person>

   If your tool makes worktrees for you, use that worktree. Make sure that it starts from the
   branch of the person.

3. Work only in your worktree. Implement the todos. Run the checks of the project. Then run
   `todoer done <name>...` in your worktree for each todo that the work does, and commit your work
   with the removed todo files. If the work needs a new todo, add it with `todoer add` in your
   worktree and commit it too.

4. Submit the pull request from the checkout of the person, not from your worktree. `--branch`
   names your branch:

       cd <checkout of the person> && todoer pr submit --branch <your branch> -n <pr name> -m "<what you changed and why>"

   - The pull request name has lowercase letters, digits and hyphens, and starts and ends with a
     letter or digit, for example `fix-login-crash`.
   - The submit prints the todos that the patch removes. If a todo is missing, run
     `todoer done <name>` in your worktree, commit, and submit with `--update`.
   - The patch has all the commits of your branch after the merge base with the branch of the
     person, squashed into one commit.
   - The command writes `.todoer/prs/<pr name>/` in the working tree of the person. It does not
     commit. The person commits the folder.
   - If your worktree has changes that are not committed, the submit stops. Commit them first.
   - Do not commit. Do not push. Do not open a pull request on GitHub or on another host.
   - The pull request does not need your branch. The person can remove your worktree and your
     branch after the submit.

5. Tell the person the pull request name and the branch that it goes into. Tell the person that
   the pull request is not committed.

## Review a pull request with the person

Review in the checkout of the branch that the pull request goes into. Read the patch, explain it,
and add the comments that the person asks for.

    todoer pr ls
    todoer pr show <pr> --summary
    todoer pr show <pr> <file>... [--context 5] [--lines 40-80]
    todoer pr comment <pr> <target> "<text>"

- `-a security,tests` shows only the files in these areas of `.todoer/areas`. `-x unit-tests`
  hides the files in these areas. Use them when the person asks to review a part of the code.
- `pr show --summary` lists the files. `pr show <pr> <file>...` shows the diff of these files only.
- `-a` and `-x` also work with `pr show` and no file.
- `pr show` shows the line numbers of the new version. Use them in the target, for example
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

       todoer pr show <pr>

   The branch of this checkout is the branch that the pull request goes into (`<into>`). Note the
   path of this checkout. The pull request and its review must be committed on `<into>`. If
   `git status` shows them as not committed, ask the person to commit them.

2. Make a new worktree and a new branch for the fix, with your tool or with git. Start it from
   `<into>`:

       git worktree add -b <branch> <folder> <into>

   If your tool makes worktrees for you, use that worktree. Make sure that it starts from `<into>`.

3. Apply the pull request in your worktree:

       todoer pr apply <pr>

   `apply` also removes the todos of the pull request in your worktree. If `<into>` moved after
   the `base` of the pull request, `apply` can write conflict markers and name the files. Resolve
   each conflict, then run `git add` on the files. Commit the result. The branch of the first work
   is not necessary. The person can remove it after a submit.

4. Do the changes that the comments ask for. A comment can name a file that the pull request does
   not change yet: change that file too. For a rename or a move, use `git mv`. If a comment is not
   clear, ask the person.

5. Commit in your worktree. Then update the pull request from the checkout of `<into>`:

       cd <checkout of into> && todoer pr submit --branch <your branch> -n <pr> --update -m "<what the pull request changes and why>"

   `-n <pr>` finds the pull request, so the name of your branch is not important. `--update`
   writes the pull request again and removes the review. The new `base` is the commit where your
   branch starts. `-m` replaces the description. Without `-m`, the update keeps the old
   description and says so. Do every comment first: todoer does not check them. If
   you do not do a comment, tell the person.

## Plan work for the person

When the person asks you to add a todo, an issue, a task or a bug, add a todo with `todoer add`.

    todoer add <name> "<what and how to check it>"

A name has lowercase letters, digits and hyphens, and starts and ends with a letter or digit, for
example `fix-login-crash`. To rename a todo, use `git mv` on the file. When the order of the work
is important, write it in the text of the todo.

## Rules

- Do not run `todoer pr apply --merge` or `todoer pr close` unless the person asks.
- Do not edit the files in `.todoer/` yourself. Use the commands.
- Do not commit the files in `.todoer/` unless the person asks. todoer never commits.
- Do not push. todoer never pushes.
- Add `--json` to `todoer ls`, `todoer show` and `todoer pr ls` for output that you parse.
