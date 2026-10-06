# A group is a name in the groups field of its todos

## Context

The person groups related todos, like "tests refactoring" or "bump some-npm-package version to
0.3.5". The person wants the progress of each group.

## Decision

**A group is a name in the `groups` list of a todo. There is no separate group file.**

- A todo can be in many groups. A group exists while one todo is in it.
- The progress of a group is the number of `done` todos of the total.
- A pull request belongs to the groups of its todos.
- A rename or a removal changes the `groups` field of every todo in the group.

## Consequences

- A group has no description and no owner. Add a group file when one is necessary.
- The page finds a group by a slug of its name in the URL. Two names with the same slug show the
  first group.
