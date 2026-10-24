# todoer - Commands

Run every command in a git project, in any checkout and on any branch. todoer reads and writes the
`.todoer/` folder of the current checkout. It does not know a main branch, and it has no settings
file. todoer never commits: you decide when to commit its changes.

For the workflow from a new todo to a merge, read the [guide](https://sobanieca.github.io/todoer/guide.html).

## Todos

| Command                                    | Does                                      |
| ------------------------------------------ | ----------------------------------------- |
| `todoer ls [--json]`                       | lists the todos                           |
| `todoer show <name> [--json]`              | prints all the data of a todo             |
| `todoer add <name> [text]`                 | adds the todo `.todoer/todos/<name>.md`   |
| `todoer done <name>...`                    | removes the todos                         |

- A todo command changes the files of the current checkout, on any branch. It does not commit.
  You decide when to commit those changes.
- A command that takes a name also takes the path of the todo file, for example
  `.todoer/todos/fix-login.md`.
- A todo has no assignee and no state. To split the work between people, keep the list of who
  does which todo outside todoer.
- `ls` shows each open pull request that removes the todo: `pr: <name>`, with the number of
  comments. `--json` adds the field `prs`.
- `show` prints the file, the open pull requests and the text of the todo. `--json` adds `file` and `text` to the fields of `ls --json`.
- `add` refuses a bad name and prints a correct name. When there is no text and stdin is
  not a terminal, `add` reads the text from stdin.
- `done` removes the todo files. Run it in the work, before the submit: then the patch of the
  pull request removes the todos (see [Pull requests](#pull-requests)).
- To rename a todo, run `git mv` on the file.
- There is no `dropped` status. To drop a todo, run `todoer done`. git history keeps the file.
- Every command that reads all the todos prints a warning for a bad name.

### Many projects

Outside a git repository, `todoer ls` lists the todos of each git project with `.todoer/todos/` in
the current folder and in its subfolders. For example, run it in `~/code`. todoer does not look
into a project, a hidden folder or a linked worktree.

todoer reads the todos of the branch that is checked out in each project. Above the todos of a
project, it shows the path of the project relative to the current folder, and the branch.
`ls --json` adds `project` (the path) and `branch` to each todo.

```
$ cd ~/code && todoer ls
todoer  main
  fix-login

work/api  feat/rate-limit
  add-rate-limit  pr: rate-limit
```

Only `ls` reads many projects. To use another command, go to the folder of the project.

## Pull requests

| Command                                                                                   | Does                                                  |
| ----------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| `todoer pr submit --branch b -n name [-m text] [--update]`                  | writes the commits of `b` as a pull request into the current branch |
| `todoer pr ls [--json]`                                                     | lists the pull requests of the current checkout       |
| `todoer pr apply <pr>`                                                      | applies the patch to the working tree, no commit      |
| `todoer pr apply <pr> --merge`                                              | merges the pull request into the working tree          |
| `todoer pr close <pr>`                                                      | removes a pull request without a merge                |

A `<pr>` is the name or the path of the folder.

- Run `submit` in the checkout of the branch that the pull request goes into. `--branch` names
  the branch of the work. For example, check out `main` and run
  `todoer pr submit --branch fix-login -n fix-login`.
- `submit` takes the commits of `--branch` after the merge base with the current branch. This is
  all the work since `--branch` started from the current branch. Commits that change only the
  folder of the pull request do not count.
- `submit` squashes these commits into one commit in `pr.patch`. The author is you. The
  message is `todoer: merge <name>`. The messages of the commits of the work are not used.
- `submit` writes `.todoer/prs/<name>/` in the working tree of the current checkout. It does not
  commit. It prints the git command that commits the folder. Commit and push the folder, so that
  other people see the pull request.
- `--branch` is necessary on each submit. It must be a local branch, and not the current branch.
- When `--branch` is checked out in a worktree, `submit` stops while that worktree has changes
  that are not committed. Commit the work first.
- The name of the folder is the id and the title of the pull request, as for a todo. `-n` gives
  the name. `-n` is necessary: todoer does not make a name. When the current checkout has an open
  pull request with that name, `submit` stops and names `-n <pr> --update`.
- The patch holds all the changes of the work, also the changes in `.todoer/`: new todos,
  changed todos, removed todos, `.todoer/areas` and the folders of other pull requests. Only the
  folder of the pull request itself, `.todoer/prs/<name>/`, is not in the patch.
- The todos of a pull request are the todos that its patch removes. Run `todoer done <name>` in
  the work and commit it. `submit`, `pr ls --json`, `pr show` and `ls` show these todos.
- A pull request keeps only the patch of its latest submit. `-n <pr> --update` replaces the patch
  and the diff of `<pr>`, and removes the review. git history keeps the old patch and the old
  review. The branch of the work can have any name: `-n` finds the pull request.
- A submit without `--update` does not change an open pull request. It stops and names
  `-n <pr> --update`. Do what the comments ask first. todoer does not check the comments.
- A pull request does not need the branch of the work after the submit. You can remove that
  branch and its worktree.
- To continue a pull request, make a new branch from the branch that holds it, and run
  `todoer pr apply <pr>` on the new branch (see [Apply](#apply)). Then update with
  `-n <pr> --update` in the checkout that holds the pull request. todoer does not make the branch
  or the worktree.
- The `base` of a submit is the merge base of `--branch` with the current branch. Thus an update
  from a new branch moves the `base` forward.
- `--update` writes the whole pull request again, as for a new one. `-m` replaces the
  description. Without `-m`, the update copies the old description, and says so.
- `ls` lists the pull requests of the current checkout, with the number of comments and the
  author.

### Apply

`pr apply` has two jobs:

- Without `--merge`, it is for follow-up work. It puts the changes in the working tree, and the
  pull request stays open. Use it to run the code, or to continue the work of the pull request.
- With `--merge`, it closes the pull request with a merge. See [Merge](#merge).

To run the code of a pull request, or to continue its work, make a new branch from the branch
that holds the pull request. Then run `todoer pr apply <pr>`.

- `apply` puts the changes of the patch in the working tree and in the index. It does not commit.
- `apply` stops when the current checkout has changes that are not committed. Changes in the
  folder of the pull request do not count.
- `apply` uses `git apply --3way`. When the branch moved after the `base` of the patch and a
  change conflicts, `apply` writes conflict markers in the files and names them. Resolve the
  conflicts, run `git add` on the files, then commit.
- When the patch does not apply for another reason, nothing changes. An example is a file that
  git does not track at the path of a new file.
- A three-way merge needs the `base` commit of the pull request. A shallow clone can lack it. Then
  `apply` and `apply --merge` stop and tell you to run `git fetch --unshallow`.
- To update the pull request, commit your changes. Then check out the branch that the pull
  request goes into, and run `todoer pr submit --branch <your branch> -n <pr> --update`. `apply`
  prints this command with the current branch. The update replaces the pull request. Its new
  `base` is the commit where your branch starts.
- `apply` removes the todos of the pull request by name, and stages the removal, as `--merge`
  does.
- Without `--merge`, `apply` does not read the review and does not remove the pull request.

### Merge

There is no approve step. To accept a pull request, merge it. Run `todoer pr apply <pr> --merge`
in the checkout of the branch that the pull request goes into. `submit` already put the folder
there. A merge closes the pull request.

- `--merge` stops when the review has a comment. A comment is resolved when
  `pr submit --update` removes the review.
- When the changes of the patch are already on the current branch, for example after a git merge
  of the branch, `--merge` does not apply the patch again.
- Else `--merge` first tries the patch on a copy of the git index. When it applies, it runs
  `git apply --3way`. A base that moved is not a problem while the patch applies. When the patch
  does not apply, nothing changes. Then update the pull request: make a new branch from the
  current branch, run `todoer pr apply <pr>` on it, resolve the conflicts and commit. Then run
  `todoer pr submit --branch <the new branch> -n <pr> --update` and merge again.
- `git apply` leaves out the todo files that the patch removes. `--merge` removes these todos by
  name. Thus an edit of a todo on the current branch does not stop the merge.
- Then `--merge` removes the folder of the pull request. It stages all these changes with
  `git add`. It does not commit.
- `--merge` prints the commit command: `git commit -m "todoer: merge <pr>"`. Other staged changes
  go into this commit too, so stage nothing else before a merge.
- `close` removes the folder without a merge. The todos stay. It does not commit.

### Commits

todoer never commits. You decide when to commit the changes, and what goes into each commit.
`submit`, `apply --merge` and `close` print the git command that commits their change. Commit and
push the todos, a pull request and a review, so that other people see them. todoer never pushes.

## Review

| Command                                                    | Does                                       |
| ---------------------------------------------------------- | ------------------------------------------ |
| `todoer pr show <pr> [--summary] [-a area,...] [-x area,...]` | the summary and the diff of all the files |
| `todoer pr show <pr> file... [--context N] [--lines 40-80]` | the diff of these files only            |
| `todoer pr comment <pr> [target] [text]`                   | adds a comment to `review.md`              |

- Review in the checkout of the branch that the pull request goes into. The folder is there.
- `show` prints the summary first: the files, the comment counts and the areas. Then it prints
  the diff with the comments. `--summary` prints only the summary.
- With files, `show` prints only the diff of these files, with no summary.
- The diff shows the old and the new line numbers. `--context N` reads the old file from the base
  and shows `N` more lines around each change. `--lines 40-80` shows only these new lines.
- Colors and `$PAGER` are on only when the output is a terminal. `NO_COLOR` turns colors off.
- The review, `review.md`, is for the latest patch only. `todoer pr submit --update` removes it,
  and the next review starts with no comments.
- `comment` writes `review.md` and does not commit it. When the review is done, commit it:
  `git add .todoer/prs/<pr>/review.md && git commit -m "Review <pr>" -- .todoer/prs/<pr>/review.md`.
- There is no verdict. To accept the pull request, merge it with `pr apply --merge`. To ask for changes, commit the review
  and ask an agent to do the comments and run `pr submit --update`.
- With one argument after `<pr>`, `comment` uses it as a target when it names a line, a file or a
  folder of the diff. Then it reads the text from stdin. Else the argument is the text of a comment
  on the whole pull request.

| Target          | Means                                      |
| --------------- | ------------------------------------------ |
| `src/a.rs:42`   | line 42 of the new version                 |
| `src/a.rs:40-45`| lines 40 to 45 of the new version          |
| `src/a.rs:old:12` | line 12 of the old version, a removed line |
| `src/a.rs`      | the whole file                             |
| `src/cli/`      | the whole folder                           |
| none            | the whole pull request                     |

A new line can be any line of any file in the new version: a file of the pull request, or a file
of the base that the pull request does not change. Use this to point to code that must change too.
An old line must be in the diff. `pr show` shows a line outside the changes with two lines
around it, and `pr show --summary` lists the files that the pull request does not change, with their
comments.

### Review with an agent

Review a pull request by talking with an agent. The agent reads the patch with `pr show`, and
adds the comments that you ask for with `pr comment`. Then commit the review.

## Code areas

An area is a named part of the code. `.todoer/areas` gives each area its paths:

```
# the last pattern that matches a path wins
security: /src/api/auth
tests: /tests
e2e-tests: /tests/e2e
unit-tests: *.test.js *_test.go
docs: /docs/ *.md
```

- Each line is `<area>: <pattern>...`. Lines that start with `#` are comments.
- The patterns use the gitignore rules. The last pattern that matches a path gives its area.
  Thus put the general areas first and the specific areas after them.
- A pattern with no `/` matches the file name in each folder. `unit-tests: *.test.js` puts
  `src/api/auth/login.test.js` in `unit-tests`, because its line comes after `security`.
- `pr show` shows each area that the pull request changes, with the number of
  files and lines. `(no area)` counts the files that no pattern matches.
- For a renamed file, the areas of the old and the new path count.
- When `.todoer/areas` is missing, the summary shows no areas.
- `-a security,e2e-tests` (`--area`) shows only the files in these areas.
- `-x e2e-tests,unit-tests` (`--exclude`) hides the files in these areas. The files with no area
  stay.
- `pr show` takes `-a` and `-x`. You can use the two together. Give the
  names with commas, or give the option more than one time. An unknown name is an error.
- A renamed file stays when one of its areas is shown.

## Other commands

| Command                   | Does                                               |
| ------------------------- | -------------------------------------------------- |
| `todoer skill [--init]`   | prints the agent workflow, or writes it as a skill |
| `todoer update`           | installs the latest release                        |
| `todoer help`             | prints the help                                    |
| `todoer version`          | prints the version                                 |

`skill --init` writes `.claude/skills/todoer/SKILL.md` and `.agents/skills/todoer/SKILL.md`. In a
git repository, it writes them in the root of the repository, also from a subfolder. Outside a git
repository, it writes them in the home folder (`~/.claude/skills/todoer/` and
`~/.agents/skills/todoer/`), so that the agents read the skill in all projects. When a file exists,
it stops and writes nothing.
