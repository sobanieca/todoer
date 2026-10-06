export const SKILL_NAME = 'todoer'

export const SKILL_TARGETS = [
  { dir: '.agents/skills', readBy: 'Codex, Cursor' },
  { dir: '.claude/skills', readBy: 'Claude Code, Cursor' },
]

export const workflowText = `# todoer - workflow for AI agents

todoer keeps the todos and the pull requests of a git project as plain files in
.todoer/ of the main checkout. A pull request is a patch file. The person
reviews it on the main branch in the todoer page, and merges it from there.

Never edit the files in .todoer/ yourself. Use the todoer commands. They find
the main checkout from any worktree of the project.

## Implement todos

1. Read each todo:

     todoer show <id>

   The output gives the title, the description, the groups, the
   dependencies, and the full path of each image. Open every image. An
   image often shows the problem.

   If the output says BLOCKED, do not start the todo. Tell the person which
   todos and pull requests it waits for. "todoer ls --ready" lists the todos
   that can start now.

2. Make a worktree for the todos and claim them. Use one command for all the
   todos that the person gave you in one request:

     todoer start <id> [<id> ...]

   The command makes the branch todoer/<ids> and a worktree from the main
   branch. Under Claude Code the worktree is in .claude/worktrees/<ids>,
   otherwise in .todoer/worktrees/<ids>. "worktreesDir" in todoer.json can
   change the folder. The command prints the path of the worktree.

   If your tool already made a worktree for this session (for example the
   EnterWorktree tool of Claude Code), run this in that worktree instead:

     todoer start <id> [<id> ...] --here

3. Work only in the worktree. Do all edits, builds and tests there. Never
   change the main checkout, and never commit to the main branch.

4. Commit your work in the worktree. Use one or more commits with clear
   messages. Run the checks of the project (tests, lint, format) first.

5. Submit the pull request from the worktree:

     todoer pr submit --title "<short title>" --message "<what you changed and why>"

   The command writes the commits as a patch into .todoer/prs/<pr id>/ of the
   main checkout. It prints the id of the pull request. Do not push the branch
   and do not open a pull request on GitHub or another host.

6. Tell the person the id of the pull request, and that they can review it
   with "todoer ui".

## Address a review

1. Read the review:

     todoer pr show <pr id>

   The output gives the worktree path and each comment with its target:

   - path:42      - line 42 of the new version of the file
   - path:old:42  - line 42 of the old version (a removed line)
   - path         - the whole file, for example "rename it" or "move it"
   - folder/      - the whole folder, for example "move it under src/"

   Add --diff to see the change that the person reviewed.

2. Go to the worktree that "todoer pr show" prints. Do every change that the
   comments ask for. For a rename or a move, use "git mv". If a comment is
   not clear, ask the person before you change the code.

3. Commit, then submit again from the worktree:

     todoer pr submit --message "<what changed since the last revision>"

   This adds the next revision (v2, v3, ...) to the same pull request.

## Plan work for the person

When the person asks you to split work into todos, use the commands:

     todoer add "<title>" --body "<what and how to check it>" --group "<group name>"
     todoer dep add <id> <id it depends on>

A group is a name for related todos, like "tests refactoring". Add a
dependency only when a todo cannot start before the other is done. Check
the plan with "todoer group show <name>" and "todoer dep show <id>".

## Rules

- Do not run "todoer pr merge" or "todoer pr close" unless the person asks.
- Do not use --force unless the person asks for it.
- Do not change the status of a todo by hand. The commands set it:
  start -> in-progress, pr submit -> review, merge -> done.
- The image limit is "maxImageKb" in todoer.json. "todoer attach" refuses a
  larger image.
- Add --json to "todoer ls", "todoer show" and "todoer pr show" for output
  that you parse.
`

export const skillContent = `---
name: ${SKILL_NAME}
description: Implement todos of the todoer tracker in a separate git worktree and submit the work as a patch-based pull request, or address the review comments of a todoer pull request. Use when asked to implement todoer todos (ids like 7f3a) or to fix a todoer pull request review.
---

${workflowText.replace(/^# todoer - workflow for AI agents\n\n/, '# todoer\n\n')}`
