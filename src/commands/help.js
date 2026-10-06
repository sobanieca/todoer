import { CONFIG_FILE, DEFAULT_MAX_IMAGE_KB, DEFAULT_PORT } from '../config.js'
import { STATUSES } from '../todos.js'
import { version } from '../version.js'

export const helpText = () => `
todoer ${version} - todos and patch-based pull requests for a git project, made for AI agents.

Usage:

  todoer init                           create ${CONFIG_FILE} and the .todoer/ folder
  todoer ui [--port <port>] [--open]    open the todoer page on this machine
  todoer ls [--status <list>] [--all] [--group <name>] [--ready]
                                        list the open todos (--ready: can start now)
  todoer add "<title>" [--body <text>] [--group <name> ...]
                                        add a todo
  todoer show <id>                      print a todo, with the paths of its images
  todoer status <id> <status>           set the status: ${STATUSES.join(', ')}
  todoer attach <id> <image>            attach a PNG, JPEG, GIF or WebP image
  todoer rm <id>                        remove a todo and its images
  todoer group ls | show "<name>"       list the groups with their progress, or one group
  todoer group add "<name>" <id> [...]  put todos in a group (a new name makes a group)
  todoer group rm "<name>" [<id> ...]   take todos out of a group, or remove the group
  todoer group rename "<old>" "<new>"   rename a group
  todoer dep add <id> <id> [...]        the first item depends on the others
  todoer dep rm <id> <id> [...]         remove dependencies
  todoer dep show <id>                  print what the item depends on and what needs it
  todoer start <id> [<id> ...] [--here] [--force]
                                        make a worktree and a branch for the todos
  todoer pr submit [--title <t>] [--message <m>]
                                        send the commits of this worktree as a pull request
  todoer pr ls [--all]                  list the open pull requests
  todoer pr show <id> [--diff]          print a pull request and its review comments
  todoer pr comment <id> <target> "<text>"
                                        add a review comment
  todoer pr merge <id> [--keep] [--force]
                                        apply the patch on the main branch
  todoer pr close <id>                  close a pull request without a merge
  todoer pr prompt <id>                 print the prompt that asks an agent to fix a review
  todoer skill [--init]                 print the workflow for AI agents, or create a skill
  todoer help | version

  --json on ls, add, show, start, pr submit, pr ls and pr show prints JSON.

Work with an AI agent:

  1. Add todos in the page (todoer ui) or with todoer add.
  2. Select todos in the page and click "Ask agent". Give the prompt to your
     agent, or let the page start the agent from "agentCommand".
  3. The agent runs todoer start, works in the worktree, commits, and runs
     todoer pr submit. The patch goes to .todoer/prs/ of the main checkout.
  4. Review the pull request in the page. Comment on a line, a file or a
     folder. Request changes, or merge it into the main branch.

Groups and dependencies:

  A group is a name, like "tests refactoring". A todo can be in many groups.
  A todo or a pull request can depend on other todos and pull requests, to
  any depth. An item waits until each todo that it depends on is done, and
  each pull request is merged. A pull request also waits for the
  dependencies of its todos. start and pr merge refuse an item that waits;
  --force skips the check. A dependency that makes a cycle is refused.
  Write pr:<id> or todo:<id> when an id is not clear.

Review targets (todoer pr comment):

  src/app.js:42         line 42 of the new version of the file
  src/app.js:old:42     line 42 of the old version, a removed line
  src/app.js            the whole file
  src/legacy/           the whole folder

Settings (${CONFIG_FILE} at the project root, every key is optional):

  maxImageKb            the largest image, in kB (default ${DEFAULT_MAX_IMAGE_KB})
  mainBranch            the branch that pull requests go to (default: the branch
                        of the main checkout)
  worktreesDir          the folder for new worktrees (default .todoer/worktrees,
                        .claude/worktrees under Claude Code)
  agentCommand          the command that runs an agent, like ["claude", "{prompt}"]
  port                  the port of todoer ui (default ${DEFAULT_PORT})

Run 'todoer skill' to read the workflow for AI agents.
`

export const help = () => console.log(helpText())
