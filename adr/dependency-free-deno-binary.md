# Plain JavaScript on Deno, with no imports from outside the repo

## Context

todoer runs on the machine of the developer, next to agents. It must be one file to install.

## Decision

**todoer is plain JavaScript, compiled with `deno compile`, with the built page inside.**

- The server uses `Deno.serve`. Git runs through `Deno.Command`.
- The tests use `test/assert.js`, not a library.
- The page is an imp project. imp builds it into `ui/dist`, and `deno compile --include` embeds it.

## Consequences

- Small helpers, such as the argument parser and the diff parser, live in `src/` with tests.
- git must be on the PATH.
