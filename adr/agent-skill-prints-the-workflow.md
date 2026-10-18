# todoer skill is the one source of the agent workflow

## Context

Agents differ: Claude Code, Codex, Cursor. The workflow must work with every agent, also when no
skill is installed.

## Decision

**`todoer skill` prints the workflow. `todoer skill --init` writes the same text as a skill.**

- The text is `src/templates/SKILL.md`.
- `--init` writes `.claude/skills/todoer/SKILL.md` and `.agents/skills/todoer/SKILL.md`.
- In a git repository, `--init` writes them in the root of the repository. Outside a git
  repository, it writes them in the home folder, so that the agents read the skill in all projects.
- The help tells an AI agent to run `todoer skill`.

## Consequences

- A change to the workflow is one change in `src/templates/SKILL.md`.
- An agent with no skill still gets the full workflow from one command.
