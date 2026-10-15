# The review is for the latest patch

## Context

Agents write the code. A person reviews a patch, and the agent does every comment in the next
submit. Revisions, comments that move to a new revision and a resolve step added work but no value.
git history keeps every old patch and every old review.

## Decision

**A pull request keeps only the patch of its latest submit. `todoer pr submit --update` replaces
the patch and removes the review. The next review starts with no comments.**

- `pr.patch` and `pr.diff` are the only patch files. There are no revision numbers.
- When the review has comments, `pr submit` stops and names `--update`. The author must do the
  comments first. todoer does not check them.
- `pr submit` stops when the review has draft comments, also with `--update`. The reviewer must send
  the review first, so todoer never deletes a review that is not finished.
- `-m` of a later submit writes the `## Update` section of `pr.md`. There the author tells the
  reviewer what changed, and why a comment was not done.
- There is no reply and no resolve. The `## Update` section is the answer to the review.
- `pr merge` stops when the review has a comment, also a draft. A comment is resolved only when
  `pr submit --update` removes the review.

## Consequences

- `pr details` and `pr comment` always use the latest patch. There is no `--rev`.
- To see an old patch or an old review, use `git log` on the folder of the pull request.
- An approval with comments cannot be merged. The author must do the comments and submit again.
