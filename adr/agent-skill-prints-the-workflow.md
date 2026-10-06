# todoer skill is the one source of the agent workflow

## Context

Agents differ: Claude Code, Codex, Cursor. The prompts from the page must work with every agent,
also when no skill is installed.

## Decision

**`todoer skill` prints the workflow. `todoer skill --init` writes the same text as a skill.**

- The prompts of the page tell the agent to run `todoer skill`.
- `--init` writes `.claude/skills/todoer/SKILL.md` and `.agents/skills/todoer/SKILL.md`.

## Consequences

- A change to the workflow is one change in `src/skill.js`.
- An agent with no skill still gets the full workflow from one command.
