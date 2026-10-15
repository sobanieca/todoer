# todoer

todoer keeps the todos and the pull requests of a git project as plain files. AI agents do the work
in git worktrees. You review their pull requests on the main branch, in a local web page.

- One binary for the command line and the page. No database, no server, no account.
- Todos are markdown files in `.todoer/todos/`. Images are checked against a size limit.
- An agent works in its own worktree, and sends its commits as a patch file.
- You review the patch on the main branch. You comment on a line, a file or a whole folder.
- A merge applies the patch on the main branch with `git am`.
- Groups collect related todos and show their progress.
- Todos and pull requests can depend on each other, to any depth, with no cycle.

todoer is not released yet. Until version `1.0.0`, any part of it can change.

> **Planned:** todoer moves from Deno to Rust, and becomes a command line tool with no web page.
> See [`todo/`](todo/) for the plan.

## Start

```sh
deno task compile          # builds ./todoer (needs deno and imp)
cd ~/code/my-project
todoer init                # writes todoer.json and .todoer/
todoer skill --init        # writes the todoer skill for Claude Code, Codex and Cursor
todoer ui --open           # opens the page on http://localhost:4507
```

1. Add todos in the page, or with `todoer add "<title>"`. Paste or drop images.
2. Select one or more todos and click **Ask agent**. Give the prompt to your agent.
3. The agent runs `todoer start <ids>`, works in the new worktree, commits, and runs
   `todoer pr submit`. The patch goes to `.todoer/prs/` of the main checkout.
4. Open the pull request in the page. Comment, then click **Request changes** or **Approve and
   merge**.

The commands and the settings are in [`docs/cli.md`](docs/cli.md). The files that todoer writes are
in [`docs/data.md`](docs/data.md).

## Settings

`todoer.json` at the project root holds the settings. Every key is optional.

```json
{
  "maxImageKb": 150,
  "mainBranch": "main",
  "worktreesDir": ".todoer/worktrees",
  "agentCommand": ["claude", "{prompt}"],
  "port": 4507
}
```

## Work on todoer

| Task                | What it does                                                         |
| ------------------- | -------------------------------------------------------------------- |
| `deno task start`   | builds the page and serves a demo project on <http://localhost:4507> |
| `deno task ui`      | builds the page and runs `todoer ui` in this repository              |
| `deno task test`    | runs the tests                                                       |
| `deno task check`   | runs `deno fmt`, `deno lint`, the tests, then imp fmt, lint and test |
| `deno task compile` | builds the `todoer` binary with the page inside                      |

The page is an imp project in `ui/`. The decisions are in [`adr/`](adr/). The rules for a change are
in [`CLAUDE.md`](CLAUDE.md).

## License

MIT. See [`LICENSE`](LICENSE).
