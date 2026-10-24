Website: https://sobanieca.github.io/todoer/

# About

- :robot: Do AI agents do your work, and do you want to give them a list of todos?
- :memo: Do you want your todos in git, next to the code that they change?
- :no_entry_sign: Do you want a tracker without an account, a server or a browser tab?
- :mag: Do you want to review the changes of an agent in the terminal, line by line?
- :package: Do you want a pull request that is one commit, which anyone can apply and change?
- :twisted_rightwards_arrows: Do you want a feature branch with its own todos, which merge with its code?
- :file_folder: Do you keep many projects in one folder and want all their todos in one list?

`todoer` keeps the todos and the pull requests of a git project as plain files. AI agents and people
work on the todos in git worktrees, and review the pull requests on the command line. It is one
binary. It has no database, no server, no account and no settings.

todoer is not released yet. Until version `1.0.0`, any part of it can change.

## How it works

- A todo is a file `.todoer/todos/<name>.md`. The name is its id. The file exists while the work is
  open. `todoer show <name>` prints all the data of a todo.
- todoer works on any branch, also a feature branch. todoer never commits: you decide when to
  commit its changes, and when to push them.
- A pull request is a patch in `.todoer/prs/`, on the branch that it goes into. You review
  it in the terminal: comment on a line, a range of lines, a file, a folder or the whole pull
  request.
- A pull request is one commit: a submit squashes the commits of the work. A merge
  (`todoer pr apply --merge`) applies this commit to the working tree of the current branch,
  removes the pull request, and stages the changes. You commit the merge.
- The patch carries all the work, also the todos: the agent runs `todoer done` in its worktree,
  and a merge removes the todo. A feature branch merges into `main` with its todo changes.
- A pull request names no branch. Anyone can put it in a working tree with `todoer pr apply`, change
  it and update it with `-n <pr> --update`.
- Code areas show how much of a pull request changes each part of the code.
- In a folder of projects, such as `~/code`, `todoer ls` lists the todos of every project, with
  its path and its checked-out branch. The other commands work in one project.

## Install

todoer is one binary for Linux and macOS (x64 and arm64).

```sh
curl -fsSL sobanieca.github.io/todoer/install.sh | bash
```

To install the latest release later, run:

```sh
todoer update
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
todoer add fix-login-crash "Steps to reproduce: ..."
git add .todoer && git commit -m "Plan the auth work"
```

`todoer skill --init` writes the skill in the root of the git repository. Outside a git repository,
it writes the skill in your home folder, for all projects. An agent with no skill can run
`todoer skill` to learn the workflow.

1. **Implement.** Ask an agent to implement `fix-login-crash` or a list of todos in a worktree. It makes a new branch from
   your branch, runs `todoer done fix-login-crash` there, and commits.
2. **Submit.** In the checkout of your branch, the agent runs
   `todoer pr submit --branch <its branch> -n fix-login-crash`. The pull
   request has all the commits since the work started from your branch, squashed into one commit.
   todoer writes it in your working tree. You commit it.
3. **Review.** Ask an agent to explain the patch and to add your comments with
   `todoer pr comment`. Or review in the terminal with `todoer pr show <pr>`. Then commit
   `review.md`.
4. **Update.** Ask an agent to do the comments. It makes a new worktree from your branch, runs
   `todoer pr apply <pr>` there, resolves any conflicts, does the comments and commits. Then it runs
   `todoer pr submit --branch <its branch> -n <pr> --update` in your checkout. This replaces the
   patch, moves the `base` forward and removes the review. The first work branch is not necessary
   after a submit.
5. **Merge.** On your branch, run `todoer pr apply <pr> --merge`, then commit. There is no approve
   step: a merge accepts the pull request and closes it. It stops while the review has comments. Without
   `--merge`, `todoer pr apply <pr>` only puts the changes in your working tree, for follow-up work.

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
