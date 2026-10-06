# Keep .todoer on an orphan branch and sync it

Put `.todoer/` on an orphan branch `todoer`, checked out as a worktree, and hide it from the main
branch with `.git/info/exclude`. Add `todoer sync` to fetch, rebase and push that branch. See
`analysis.md` for the reasons.
