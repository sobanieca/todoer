# Todos and pull requests depend on each other, with no cycle

## Context

Some work must wait for other work. A todo can wait for a pull request of another team member, and a
pull request can wait for a todo.

## Decision

**`dependsOn` in a todo or in `pr.md` lists `todo:<id>` and `pr:<id>` references, with no depth
limit and no cycle.**

- An item waits until each todo that it depends on is `done` and each pull request is `merged`.
- A pull request also waits for the dependencies of its todos.
- todoer refuses a dependency that makes a cycle, and a pull request that depends on its own todo.
- `todoer start` and `todoer pr merge` refuse an item that waits, unless `--force`.
- New todo ids and pull request ids never collide, so a bare id is clear in most commands.
- A removed todo leaves the `dependsOn` lists of the other items.

## Consequences

- The person can plan work in any depth, and agents start only the work that can start.
- A dependency on a closed pull request never completes. The person must remove it.
