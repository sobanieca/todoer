# todoer — analysis

A git-native, offline-first todo tracker with patch-based code review, built for teams where some members are AI agents.

## Goals

- Work fully offline. Sync happens through git itself, and every team machine keeps a full copy.
- Store todos, PRs and reviews as plain markdown and patch files that both humans and agents can read.
- Encourage trunk-based work: PRs go straight to `main` and get reviewed by other people or agents.
- Support GitHub `CODEOWNERS`, plus a todoer ownership file that can also name agents.
- Provide a local web UI for browsing todos and reviewing PRs.
- When run from a parent directory, find every git repo below it and list them as projects.

## Prior art

- **Backlog.md**: the closest match. Markdown tasks in the repo, a CLI, a kanban web UI and an agent focus.
- **git-bug**: issues stored as git objects.
- **git-appraise**: code reviews stored in git notes.
- **Radicle**: peer-to-peer issues and patches.

What sets todoer apart: patch-based PRs, agent reviewers, ownership rules that can name agents, and a view across many repos at once.

## Key design decisions

### 1. Store the data on its own orphan branch

If `.todoer/` were committed to `main`, every status change would be a commit to main, and tracker state would differ per branch (a todo marked done on `feature-x` would still be open on `main`).

Instead:

- `.todoer/` is a git worktree of an orphan branch, `todoer`.
- `.todoer/` is hidden from main through `.git/info/exclude`, so nothing gets committed.
- `todoer sync` fetches, rebases and pushes that branch.

This gives offline work and a full copy on every machine for free, and keeps code history clean.

### 2. Keep status in frontmatter, not in folders

With status folders (`new/`, `in-progress/`), two people moving the same todo to different statuses produce two clean renames. Git reports no conflict, and the todo ends up duplicated. A `status:` line in frontmatter turns the same situation into a real conflict that git reports. The CLI and UI can still show statuses as folders or columns.

### 3. Store assignments in frontmatter, not as symlinks

Symlinks break on Windows and keep a second copy of who's assigned. Use `assignee:` in frontmatter and have `todoer ls` filter on it.

### 4. Use IDs that can't collide offline

Sequential numbers collide when two people create todos offline. Use a short random or hash prefix, e.g. `7f3a-login-crash.md`, and refer to items as `7f3a`.

### 5. Identify users by git email, not name

Names collide and get spelled differently. `team.yml` maps people and agents to their emails and aliases:

```yaml
members:
  adam:       { emails: [adam@x.com], type: human }
  john:       { emails: [john@x.com], type: human }
  review-bot: { type: agent, runner: "claude -p" }
```

### 6. Send PRs as `git format-patch` series

- `format-patch` keeps author, commit messages and multiple commits. `git am` applies them.
- Revisions sit side by side (`v1.patch`, `v2.patch`), mailing-list style, so reviewers can see what changed between versions.
- Each PR records its base SHA, so todoer can run `git apply --check` and warn when a patch is stale.

### 7. Ownership

- Parse `.github/CODEOWNERS` when it exists.
- `.todoer/owners` uses the same syntax but can also name agents and teams, e.g. `/api/** @adam @review-bot`.
- `config.yml` holds the approval rules (how many approvals a merge needs).

### 8. Images

Before storing an image:

1. Scale the long edge down to about 1600px.
2. Convert to WebP.
3. Lower the quality step by step until the file is ≤250 kB.

Even at that size, images stay in git history forever (see Git LFS below).

## Data structure

```
.todoer/                      # worktree of orphan branch `todoer`
  config.yml                  # statuses, labels, image limits, approval rules, ui settings
  team.yml                    # people and agents
  owners                      # CODEOWNERS syntax + agents
  templates/
    todo/{bug,feature,maintenance}.md
    pr.md
  todos/
    7f3a-login-crash.md
  assets/
    7f3a/1.webp
  prs/
    k2m9-add-more-items/
      pr.md                   # status, base SHA, author, reviewers, linked todos
      v1.patch
      v2.patch
      reviews/
        john.v2.md
        review-bot.v2.md
```

Example todo:

```markdown
---
id: 7f3a
title: Login crashes on empty password
status: in-progress
assignee: adam
labels: [bug]
created: 2026-10-04
---
Steps to reproduce...
![](../assets/7f3a/1.webp)
```

## CLI

| Command | Does |
|---|---|
| `todoer init` | Creates the orphan branch and worktree and adds `.todoer/` to the exclude file |
| `todoer ls [--all] [--status x]` | Lists open todos assigned to me by default |
| `todoer add [bug\|feature\|…]` | Creates a todo from a template and opens `$EDITOR` |
| `todoer claim <id>` | Sets assignee to me and status to in-progress; syncs first, to reduce double-claims |
| `todoer attach <id> <img>` | Resizes and stores an image |
| `todoer pr add [--todo id]` | Turns the current branch or worktree's commits since main into a patch plus `pr.md` |
| `todoer pr update <id>` | Adds the next patch revision |
| `todoer pr comment <id> <file:line> "text"` | Adds an inline review comment without the UI |
| `todoer pr review <id> [--agent review-bot]` | Prints review context, or runs the configured agent, which writes a file to `reviews/` |
| `todoer pr merge <id>` | Checks approvals, runs `git am` onto main, records the merged SHA, closes linked todos |
| `todoer sync` | Fetches, rebases and pushes the data branch |
| `todoer ui [--port]` | Starts the local web UI; run from a parent directory, it finds all repos (with depth limits and skip lists) |

All commands support `--json` so agents can use them reliably. An MCP server can come later as a thin wrapper around the same commands.

## Agent workflow

1. Agent runs `todoer claim 7f3a`.
2. It works in `git worktree add ../wt-7f3a`.
3. It runs `todoer pr add --todo 7f3a`.
4. A reviewer agent and/or a human reviews.
5. The agent posts revisions with `todoer pr update`.
6. Someone merges.

Everything is in plain files, so an agent can read the full context without API access.

## UI

### Architecture

- **One binary:** `todoer ui` starts a server on `localhost` with the frontend assets compiled into the binary. It needs no CDN or web fonts, so it works fully offline.
- **Shared core:** the CLI and the UI call the same internal code and read and write the same `.todoer` files. There is no separate database. Each UI action (status change, comment, approve) becomes one commit on the `todoer` branch, authored as the local git user.
- **Live updates:** the server watches `.todoer/` for changes. When an agent or the CLI writes a file, open browser tabs refresh on their own.
- **Sync:** a sync button runs `todoer sync`, and an optional auto-sync (`ui.autoSync`) can run whenever the machine is online. When there are conflicts, the UI shows them instead of failing silently.
- **Multi-repo:** run from a parent directory, the UI lists every repo it found, with counts of open todos and of PRs waiting for my review.

### Views

| View | Contents |
|---|---|
| Projects | All found repos, my open todos, PRs awaiting my review |
| Todos | Board (columns by status) and list, with filters for assignee, label and status; create from a template |
| Todo detail | Rendered markdown and images, editing, image paste/drop (auto-resized), linked PRs |
| PR list | Status, author, revision, approvals compared with what's required |
| PR review | File tree, side-by-side or unified diff, inline comments, a selector for v1/v2/…, and a diff between revisions |
| PR actions | Approve / request changes, run a reviewer agent, merge when the approval rules pass |

**Diff context:** the patch file only holds a few lines around each change. To let reviewers expand the surrounding code, load the full files from the local repo with `git show <base>:<path>`. Every clone has the base commit, so this works offline.

### Review storage

Use one file per reviewer per revision. Each person only ever writes to their own files, so two people reviewing at once never cause a merge conflict. Replies also go in the replier's own file and point back to the original comment by its ID; the UI assembles the files into threads.

```markdown
---
reviewer: john
revision: 2
verdict: changes-requested   # approved | changes-requested | comment
---
Overall looks good, one concern about error handling.

### c-8k2f · src/api/items.ts:42
> const items = await db.query(sql)
Unhandled rejection here.

### c-9x1a · reply-to: c-3m7p (adam, v1)
Fixed in v2.
```

Agents write the same markdown, so there's no separate API for them, and the files are easy for both humans and agents to read.

**Comments when a new revision lands:** each comment records its file, line and a hash of the quoted code. In the next revision, the UI looks for that code snippet again. If it finds it, the comment moves to the new position; if not, the comment is marked **outdated**, as GitHub does.

### Stack

Go with `embed`, plus a small frontend (Svelte or Preact) built into the binary. This keeps the single-binary, offline goal.

## Git LFS

Git LFS (Large File Storage) is a git extension for big binary files. Git stores only a small text pointer to each file, and the real content lives on a separate LFS server (GitHub, GitLab, Bitbucket and others provide one). A clone downloads only the files needed for the commit you check out.

It's a poor default for todoer:

- **It needs a server.** LFS files have to be uploaded to and downloaded from an LFS-capable remote. That clashes with "fully offline" and with syncing directly between team machines.
- **Files can go missing.** If someone clones without fetching the LFS content, they see pointer text instead of images.
- **It's an extra install.** Every team member, and every agent environment, needs it set up.

**Decision:** keep images as normal git files and rely on the 250 kB cap. A few hundred screenshots add only tens of MB of history. Offer LFS as an opt-in setting for teams that attach many images and already use an LFS host.

## MVP roadmap

1. Core: `init`, `add`, `ls`, `claim`, `sync`, with the orphan-branch storage and frontmatter format.
2. PRs: `pr add`, `pr update`, `pr review`, `pr merge` with `format-patch` and `am`.
3. Read-only UI: projects, todo list and board, PR diff viewer.
4. Editing todos in the UI and creating them from templates.
5. Inline review comments, verdicts and threads (UI and `pr comment`).
6. Ownership file, approval rules, merge checks.
7. Image handling (resize, paste/drop in the UI).
8. Comparing revisions, carrying comments forward to new revisions.
9. Triggering agent reviews, auto-sync, MCP server.

## Open questions

1. **Orphan branch, or commit `.todoer/` to main?** The recommendation is the orphan branch. Committing to main is simpler but couples tracker state to code branches.
2. **Do code patches need to work without a shared remote?** If everyone can push branches anyway, patches are mainly for review history and portability, and keeping that history is still a good reason.
3. **Should agents trigger reviews on their own** when a PR appears, or only when a human runs `pr review`?
4. **Should the UI allow merging**, or only reviewing, with the actual merge (`git am` onto main) done from the CLI? Merging changes the code repo, not just the `todoer` branch, so keeping it in the CLI is a reasonable safety line for the MVP.
