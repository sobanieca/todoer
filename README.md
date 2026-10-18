Website: https://sobanieca.github.io/todoer/

# About

- :robot: Do AI agents do your work, and do you want to know which agent works on which todo?
- :memo: Do you want your todos in git, next to the code that they change?
- :no_entry_sign: Do you want a tracker without an account, a server or a browser tab?
- :mag: Do you want to review the changes of an agent in the terminal, line by line?
- :package: Do you want a pull request that is one commit, which anyone can apply and change?
- :bar_chart: Do you want to see the progress of a feature that has many todos?
- :file_folder: Do you keep many projects in one folder and want all their todos in one list?

`todoer` keeps the todos and the pull requests of a git project as plain files. AI agents and people
claim todos, work in git worktrees, and review the pull requests on the command line. It is one
binary. It has no database, no server, no account and no settings.

todoer is not released yet. Until version `1.0.0`, any part of it can change.

## How it works

- A todo is a file `.todoer/todos/<name>.md`. The name is its id. The file exists while the work is
  open.
- todoer works on any branch, also a feature branch. `claim` commits the todo on the current
  branch, so you can push it. The other todo commands do not commit.
- A pull request is a patch in `.todoer/prs/`, committed on the branch that it goes into. You review
  it in the terminal: comment on a line, a range of lines, a file, a folder or the whole pull
  request.
- A pull request is one commit: a submit squashes the commits of the work. A merge applies this
  commit on the current branch with `git am`. The same commit removes the pull request and its
  todos.
- A pull request names no branch. Anyone can put it in a working tree with `todoer pr apply`, change
  it and update it with `-n <pr> --update`.
- Labels show the progress of related todos. Todos depend on each other, with no cycle. A todo can
  also depend on a label: it waits for every todo of the label.
- Label rules tell in plain words which todos must have a label. The agent reads them and adds the
  labels to a new todo.
- Code areas show how much of a pull request changes each part of the code.
- In a folder of projects, such as `~/code`, `todoer ls` lists the todos of every project, with
  its path and its checked-out branch. The other commands work in one project.

## Install

todoer is one binary for Linux and macOS (x64 and arm64).

```sh
curl -fsSL sobanieca.github.io/todoer/install.sh | bash
todoer update              # installs the latest release later
```

The script installs todoer to `/usr/local/bin`, for all users. If that folder is not writable, the
script stops with an error. Then run it with `sudo`:

```sh
curl -fsSL sobanieca.github.io/todoer/install.sh | sudo bash
```

Or set your own folder with `INSTALL_DIR`. The folder must be on your `PATH`:

```sh
curl -fsSL sobanieca.github.io/todoer/install.sh | INSTALL_DIR=~/.local/bin bash
```

Or build it: `cargo build --release`, then copy `target/release/todoer` to a folder on your `PATH`.

## Start

```sh
cd ~/code/my-project
todoer skill --init        # writes the todoer skill for Claude Code, Codex and Cursor
todoer add fix-login-crash -l auth "Steps to reproduce: ..."
git add .todoer && git commit -m "Plan the auth work"
```

1. **Claim.** Ask an agent to claim `fix-login-crash`, a list of todos or a label. It runs
   `todoer claim`, which sets you as the assignee and commits the todo on your branch. Push it when
   you want.
2. **Implement.** Ask the agent to implement the todos in a worktree. It makes a new branch from
   your branch, and commits.
3. **Submit.** The agent runs
   `todoer pr submit --branch <your branch> -n fix-login-crash --todo fix-login-crash`. The pull
   request has all the commits since the work started from your branch, squashed into one commit.
   todoer commits it on your branch.
4. **Review.** Ask an agent to explain the patch and to add your comments with
   `todoer pr comment`. Or review in the terminal with `todoer pr review <pr>`. Then commit
   `review.md`.
5. **Update.** Ask an agent to do the comments. It makes a new worktree at the `base` of the pull
   request, restores the commit with `git am`, does the comments and runs
   `todoer pr submit --branch <your branch> -n <pr> --update`. This replaces the patch and removes
   the review. The first work branch is not necessary after a submit.
6. **Merge.** On your branch, run `todoer pr merge <pr>`. There is no approve step: a merge accepts
   the pull request. It stops while the review has comments.

The guide is at https://sobanieca.github.io/todoer/guide.html. The commands are in
[`docs/cli.md`](docs/cli.md). The files that todoer writes are in [`docs/data.md`](docs/data.md).

## Work on todoer

| Task                     | What it does                                       |
| ------------------------ | -------------------------------------------------- |
| `cargo todoer [args]`    | runs todoer from source                            |
| `cargo verify`           | runs `cargo fmt --check`, clippy and all the tests |
| `cargo update-snapshots` | accepts new CLI output in `tests/snapshots`        |

The decisions are in [`adr/`](adr/). The planned work is in [`.todoer/todos/`](.todoer/todos/). The rules for a
change are in [`AGENTS.md`](AGENTS.md).

## License

MIT. See [`LICENSE`](LICENSE).
