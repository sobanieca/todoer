# The page is an imp app on localhost

## Context

The page can change files and run git. Another web site in the same browser must not use it.

## Decision

**`todoer ui` listens on `127.0.0.1` and `::1` only, and checks every request.**

- The Host header must be `localhost`, `127.0.0.1` or `[::1]` with the port.
- A request that changes data must have the Origin of the page.
- The responses have a strict Content Security Policy.
- The server watches `.todoer/` and sends a `change` event. The page then loads its data again.

## Consequences

- The page shows the changes of agents and of the CLI at once.
- The page cannot be used from another machine.
