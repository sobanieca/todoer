# todoer is one Rust binary with few dependencies

## Context

`jsonr` and `sqlr` are Rust binaries with the same layout. todoer was a Deno binary with a web page.
The new design has no page, so a command line tool is enough.

## Decision

**todoer is one Rust binary. The only crate is `ureq`.**

- The argument parser, the colors, the JSON writer, the front matter parser, the diff parser and the
  gitignore pattern matcher are small modules in the crate, with tests.
- `ureq` downloads the releases for `todoer update`.
- todoer runs git as a child process. It does not link a git library.
- The layout is the same as in `jsonr` and `sqlr`: an `xtask` crate, cargo aliases, snapshot tests.

## Consequences

- The binary starts fast and has no runtime to install.
- A new helper that a crate would give is code in the crate, with a unit test.
