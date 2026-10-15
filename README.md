Website: https://sobanieca.github.io/todoer/

# todoer

todoer keeps the todos and the pull requests of a git project as plain files. AI agents and people
claim todos, work in git worktrees, and review the pull requests on the command line.

- One binary. No database, no server, no account.
- A todo is a markdown file with a `status` field in its front matter. Its path is its id.
- A claim is a commit on the main branch, so every agent sees at once who works on a todo.
- A pull request is a patch file in `.todoer/prs/` on the main branch. You review it in the
  terminal: comment on a line, a range of lines, a file, a folder or the whole pull request.
- A merge applies the patch on the main branch with `git am`, and sets the todos to `done`.
- Labels show the progress of related todos. Todos depend on each other, with no cycle.
- Code owners and teams filter the files of a review.

todoer is not released yet. Until version `1.0.0`, any part of it can change.

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
todoer add -t "Fix the login crash" -p todo -l auth "Steps to reproduce: ..."
git add todo && git commit -m "Plan the auth work"
```

1. Ask an agent to work on `todo/fix-the-login-crash.md`. The agent runs `todoer claim`, makes a
   worktree, commits, and runs `todoer pr submit`.
2. Review with an agent: ask it to explain the patch and to add your comments on lines of the
   change. The agent runs `todoer pr details <slug>` and `todoer pr comment <slug> <target>`.
   Or review in the terminal: `todoer pr review <slug>`.
3. Send the review: `todoer pr request-changes <slug>` or `todoer pr approve <slug>`.
4. The agent does what the comments ask and runs `todoer pr submit --update`. This replaces the
   patch and removes the review. Review the new patch from the start.
5. Merge: `todoer pr merge <slug>`. It stops while the review has comments.

The commands are in [`docs/cli.md`](docs/cli.md). The files that todoer writes are in
[`docs/data.md`](docs/data.md).

## Settings

`todoer.json` at the project root holds the settings. Every key is optional.

```json
{
  "mainBranch": "main"
}
```

## Work on todoer

| Task                     | What it does                                       |
| ------------------------ | -------------------------------------------------- |
| `cargo todoer [args]`    | runs todoer from source                            |
| `cargo verify`           | runs `cargo fmt --check`, clippy and all the tests |
| `cargo update-snapshots` | accepts new CLI output in `tests/snapshots`        |

The decisions are in [`adr/`](adr/). The planned work is in [`todo/`](todo/). The rules for a
change are in [`AGENTS.md`](AGENTS.md).

## License

MIT. See [`LICENSE`](LICENSE).
