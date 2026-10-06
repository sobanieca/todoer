# The page opens a dependency tree one level at a time

## Context

Dependencies have no depth limit. A graph where two paths reach the same item grows fast when the
page draws every path.

## Decision

**The page builds the tree rows of the open levels only, and it draws a shared item once.**

- A closed row has one placeholder child, so the person can open it. Opening it builds the next
  level.
- An item that the tree already shows says "see above" and does not open again.
- "Expand all" opens every level. The tree then has at most one full row per item.
- The tree scrolls in its own box, in both directions, so a deep tree does not widen the page.
- A selected row shows a link to its item. The page of that item shows its own tree.

## Consequences

- A deep or a wide graph stays fast and readable.
- The CLI prints the same tree with `todoer dep show`, with the same "see above" rule.
