---
title: Review a pull request on the command line
status: todo
labels: [cli-reshape]
dependsOn: [todo/patch-pull-requests.md]
---

The review commands read `v<N>.diff` only. They need no worktree, so agents can use them too.

## Summary

`todoer pr summary <pr>`

```
port-cli  Port the CLI parser to Rust   open  v2  by agent-1
base 3f2a91c   todos: todo/rust/port-cli.md   v1: changes-requested

 M  src/cli/args.rs            +120  -4   2 comments  @backend
 A  src/cli/mod.rs              +40       -           @backend
 R  src/old.js -> src/lib.js     +2  -2               (no owner)
 D  test/args.test.js                -88              @qa
4 files, +162 -94    3 commits    1 draft comment
```

## Details

`todoer pr details <pr> [file...] [--rev N] [--context N] [--lines 40-80]`

```
src/cli/args.rs  (M, +120 -4)
   40   40 |   let items = load();
   41      | - let b = 2;
        41 | + let b = 3;
           |   [c1] sobanieca: use the constant
```

- With no file, it prints all the files.
- It shows the old and the new line numbers, and the comments with their ids.
- Colors and `$PAGER` only when the output is a terminal. `NO_COLOR` turns colors off.
- `--context N` reads the old file with `git show <base>:<path>` and applies the diff to it.

## Comments

`todoer pr comment <pr> [target] "<text>"`

| Target            | Means                                    |
| ----------------- | ---------------------------------------- |
| `src/a.rs:42`     | line 42 of the new version               |
| `src/a.rs:40-45`  | lines 40 to 45 of the new version        |
| `src/a.rs:old:12` | line 12 of the old version, a removed line |
| `src/a.rs`        | the whole file                           |
| `src/cli/`        | the whole folder                         |
| none              | the whole pull request                   |

- A line must be in the diff, also a context line. The comment keeps the text of its line.
- With no text, todoer reads the text from stdin.
- `todoer pr reply <pr> <comment-id> "<text>"` answers a comment.

## Verdict

- A new comment is a draft: a change in `review.v<N>.md` that is not committed.
- `todoer pr approve <pr>` and `todoer pr request-changes <pr> [-m <text>]` set the verdict and
  commit the review file. This gives one commit for each review.
- `todoer pr review <pr>` with no interactive mode prints the summary and the details of all files.

## ADRs

Update `review-comment-targets` with the line ranges and the draft comments.
