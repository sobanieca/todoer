We need to change look&feel for UI. Currently it's too generic. We need something modern

We need to simplify concept and UI/UX. There should be only small set of commands needed. UI is too complex

On items list, buttons are misaligned. Most likely it's an issue in imp-ui project. Fix has go there if that's the case.

When adding new group, one has to press Enter first to get add button enabled. Why?

I want todoer to be very simple in concepts:

todoer ls (list active items)
todoer add -t some-title "This is sample todo" (should work with pipes)
 - adds task to current work dir. Eventually should allow -p or --path as param

 no need for tasks id or prs. Path relative to repo root is the id.

todoer complete -p "path to todo"

status inside frontmatter in markdown file
assignee as well
teams and codeowners should be defined in teams folder inside .todoer
