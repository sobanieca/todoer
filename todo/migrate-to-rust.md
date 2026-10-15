---
title: Migrate from Deno to Rust
status: todo
labels: [rust-migration, cli-reshape]
---

Replace the Deno version with a Rust version. Use the same layout as `jsonr` and `sqlr`.

todoer is unreleased. Do not port the code one to one. Port only the parts that the new design
keeps. The new design is in the other files of `todo/`.

## Layout

- `Cargo.toml` with `edition = "2024"`, `publish = false`, and a release profile with `lto`,
  `codegen-units = 1` and `strip`.
- An `xtask` crate and cargo aliases in `.cargo/config.toml`: `cargo todoer`, `cargo verify` (fmt
  check, clippy with `-D warnings`, all tests) and `cargo update-snapshots`.
- Unit tests next to the code. `tests/cli.rs` runs the binary in a temporary git repository and
  compares the output with `tests/snapshots/*.snap`.
- `src/help.txt` for the help. `src/templates/SKILL.md` for the agent skill.
- `AGENTS.md` replaces `CLAUDE.md`. Keep the rules on prose style, ASCII only and no comments.
  Rust module names use underscores, so the hyphen rule applies only to other files.
- A publish workflow that builds Linux (musl) and macOS binaries on each push to `main`.
  `install.sh` and `todoer update` download the release assets.

## Dependencies

Keep the crate list minimal, as in `jsonr` and `sqlr`. Write the argument parser, the colors, the
front matter parser and the diff parser in the crate. Use `libc` for the raw terminal mode, as
`sqlr` does in `src/prompt.rs`. Run git as a child process.

## Port

- Keep: finding the main checkout, `todoer.json`, front matter, the diff parser, the review file
  format, the patch submit and the merge, `todoer skill`.
- Do not port: the page (`ui/`, `ui-server`, `ui-api`, `events`, `static-files`), `groups`, the id
  based dependencies, `images`, `attach`, `start`, `pr prompt`.

## Remove

- `deno.json`, `main.js`, `src/*.js`, `test/`, `ui/`, `dev/`.
- The ADRs `dependency-free-deno-binary`, `local-page` and `lazy-dependency-tree`. Add an ADR for
  the Rust binary and its dependency rule.
