# Code areas show the scope of a pull request

## Context

A large pull request touches many parts of the code. A reviewer wants to know which parts change,
and how much, before reading the diff. Code owners and teams did this with filters, but they need
team files and a person for each rule. A name for each part of the code is sufficient.

## Decision

**`.todoer/areas` names the areas of the code. `pr show` shows each area that a pull request
changes, with the number of files and lines. `-a <area,...>` shows only the files of these areas,
and `-x <area,...>` hides them.**

- Each line is `<area>: <pattern>...`. Lines that start with `#` are comments.
- The patterns use the gitignore rules. The last pattern that matches a path wins, as in CODEOWNERS.
- `(no area)` counts the files that no pattern matches.
- For a renamed file, the areas of the old and the new path count.
- A pattern with no `/` matches the file name in each folder. Thus `unit-tests: *.test.js` puts the
  test files in an area, also inside the folder of another area.
- `--area` (`-a`) and `--exclude` (`-x`) filter `summary`, `details` and `review`. A file stays
  when one of its areas is in `-a` (or `-a` is not given) and is not in `-x`. The files with no
  area stay with `-x`. An unknown area name is an error.
- There are no code owners and no teams. An area has no members.

## Consequences

- A file is in one area for each path. Put the general areas first and the specific areas after
  them.
- When `.todoer/areas` is missing, the summary shows no areas.
- Approval rules before a merge are not part of this decision.
