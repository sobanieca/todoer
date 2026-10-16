# A merge is the approval

## Context

`pr approve` and `pr request-changes` wrote a verdict in `review.md` and committed it. An approval
had no effect: the person merged after it. A request for changes only told an agent to do the
comments. The person tells the agent this directly. The verdict was one more step and one more
state to keep correct.

## Decision

**A review has no verdict. To accept a pull request, the person merges it. To ask for changes, the
person commits the review and gives it to an agent.**

- `review.md` holds only the comments. It has no front matter.
- `pr comment` writes `review.md` and does not commit it. The person commits it with git when the
  review is done.
- `pr ls`, `pr summary` and `progress` show the number of comments.

## Consequences

- A review is one commit that the person makes.
- `pr merge` still stops while the review has a comment.
