---
status: todo
dependsOn: [todo/labels-and-progress.md]
---

A person wants new todos to get the correct labels without a manual step. A label rule tells in
plain words which todos must have a label. todoer does not read the words. The agent reads the
rules and adds the labels.

## Files

A rule is a markdown file `.todoer/labels/<label>.md`. The file name is the label. The body is the
rule.

```markdown
Add this label to a todo that changes the login, the sessions or the tokens.
Do not add it to a todo that only changes the docs of the login.
```

- The file name must match `^[a-z0-9-]+$`, as a label.
- A rule can exist before a todo has its label. `todoer labels` shows the label with `0 todos`.
- Commit the rules as other files. Same as `.todoer/teams/`.

## Commands

| Command                                 | Does                                              |
| --------------------------------------- | ------------------------------------------------- |
| `todoer label rule <label> [text]`      | writes the rule; the text can come on stdin       |
| `todoer label rule <label> --rm`        | removes the rule                                  |
| `todoer label rules [--json]`           | prints each label with its rule                   |

- `todoer add` prints `check the label rules: todoer label rules` after the path, when a rule
  exists.
- `todoer check` reports a rule file with a bad name, or an empty rule.
- `todoer labels --json` adds a `rule` field.

## Skill

Update `src/templates/SKILL.md`. `todoer skill` prints it, and `todoer skill --init` writes it.
Add these steps to "Plan work for the person":

1. After `todoer add`, run `todoer label rules`.
2. Read each rule, and compare it with the title and the text of the new todo.
3. Run `todoer label add <path> <label>` for each rule that the todo matches.
4. Tell the person which labels you added and why. If a rule is not clear, ask the person.

The agent also follows the rules when it changes the text of a todo. It does not remove a label
that the person added.

## Done when

- The commands, the `check` problems and the `--json` field work and have tests.
- `src/help.txt`, `docs/cli.md`, `docs/data.md`, the skill and the README show the rules.
- An ADR records that todoer stores the rules and the agent applies them.
