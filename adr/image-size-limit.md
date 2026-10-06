# todoer refuses an image above maxImageKb

## Context

Images stay in the files of the project, and in its git history when the person commits `.todoer/`.
Large images make the repository slow.

## Decision

**todoer refuses an image that is larger than `maxImageKb` (default 150 kB). It does not resize.**

- The page checks the size before the upload. The server checks it again.
- todoer accepts PNG, JPEG, GIF and WebP. It checks the first bytes of the file, not the name.
- `todoer.json` changes the limit for a project.

## Consequences

- A person must make a large screenshot smaller before they attach it.
- An automatic resize is planned, see `todo/shrink-large-images.md`.
