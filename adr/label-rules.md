# todoer stores the label rules, and the agent applies them

## Context

A person wants new todos to get the correct labels without a manual step. The person can tell in
plain words which todos must have a label. An agent adds most of the todos, and it can read plain
words. todoer cannot.

## Decision

**A label rule is the file `.todoer/labels/<label>.md`. The file name is the label, and the body
is the rule in plain words. todoer stores and lists the rules. The agent reads them and adds the
labels.**

- `todoer labels rule <label> [text]` writes the rule. `--rm` removes it.
- `todoer labels` prints each rule below its label. `--json` adds the field `rule`.
- A rule can exist before a todo has its label. Then the label exists with `0 todos`.
- `labels rm` removes the rule, and `labels mv` renames it. `labels mv` stops when the two labels
  have a rule.
- `todoer add` tells the agent to check the rules when a rule exists.
- `todoer check` reports a file with a bad name, and an empty rule.
- The skill tells the agent to read the rules before `todoer add`, and to give `-l` for each rule
  that matches. It also applies the rules when it changes a todo. The agent does not remove a
  label that the person added.
- The description of the skill names todos, issues, tasks and bugs, so an agent loads the skill
  when the person asks for any of them.

## Consequences

- todoer has no matcher and no rule language. The rules can say anything that a person can say.
- Two agents can read a rule in different ways. The agent tells the person which labels it added
  and why, and asks when a rule is not clear.
- A todo that a person adds without an agent gets no labels from the rules.
