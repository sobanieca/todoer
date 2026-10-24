# An update replaces the whole pull request

## Context

A pull request kept the description of its first submit. An
`## Update` section held the answer of the author to the review. The author had to think about the
old comments and the old text. An update by another person mixed old data and new data. git history
keeps every old version of the folder.

## Decision

**`todoer pr submit -n <pr> --update` replaces the whole pull request. todoer writes `pr.md`
and `pr.patch` again, as for a new pull request, and removes the review.**

- `pr.md` gets the author and the base of this submit.
- `-m` replaces the description.
- Without `-m`, the update copies the description of the old `pr.md`, and says so.
- The todos of the pull request are the todos that the new patch removes.
- There is no `## Update` section, no reply and no resolve.
- `pr.patch` is the only patch file. There are no revision numbers.
- Without `--update`, a submit never changes an open pull request.
- `pr apply --merge` stops when the review has a comment. Only an update removes the review.

## Consequences

- `pr show` and `pr comment` always use the latest patch. There is no `--rev`.
- To see an old patch, an old description or an old review, use `git log` on the folder of the pull
  request.
- An update by another person keeps the description, unless that person replaces it.
- To remove a todo from a pull request, restore the todo file on the branch of the update.
- The author of the pull request is the person of the latest submit.
- A review with comments cannot be merged. Update the pull request first.
