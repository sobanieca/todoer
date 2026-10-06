import { parseArgs } from '../src/args.js'
import { defaultWorktreesDir, parseConfig } from '../src/config.js'
import { lineText, parseDiff } from '../src/diff.js'
import { formatDocument, listField, parseDocument } from '../src/frontmatter.js'
import { slugify } from '../src/ids.js'
import { checkImage, imageExtension } from '../src/images.js'
import { formatReview, formatTarget, parseReview, parseTarget } from '../src/reviews.js'
import { agentCommandFor, implementPrompt } from '../src/agent.js'
import { assertEquals, assertThrows } from './assert.js'
import { pngOfSize } from './fixtures.js'

Deno.test('parses flags, aliases and values', () => {
  assertEquals(parseArgs(['pr', 'submit', '-m', 'text', '--title=T', '--json']), {
    positional: ['pr', 'submit'],
    flags: { message: 'text', title: 'T', json: true },
  })
  assertThrows(() => parseArgs(['--nope']), /unknown option --nope/)
  assertThrows(() => parseArgs(['--port']), /--port needs a value/)
})

Deno.test('reads todoer.json and refuses a bad key or value', () => {
  assertEquals(parseConfig('{"maxImageKb": 300}'), { maxImageKb: 300 })
  assertThrows(() => parseConfig('{"maxImageKB": 300}'), /unknown key "maxImageKB"/)
  assertThrows(() => parseConfig('{"maxImageKb": -1}'), /"maxImageKb" must be/)
  assertThrows(() => parseConfig('{"worktreesDir": "/tmp"}'), /relative to the project root/)
  assertThrows(() => parseConfig('[1]'), /must hold an object/)
  assertThrows(() => parseConfig('{'), /not valid JSON/)
})

Deno.test('puts worktrees under .claude/ only under Claude Code', () => {
  assertEquals(defaultWorktreesDir({ CLAUDECODE: '1' }), '.claude/worktrees')
  assertEquals(defaultWorktreesDir({}), '.todoer/worktrees')
})

Deno.test('writes and reads a document with front matter', () => {
  const text = formatDocument(
    { id: 'ab12', title: 'A: b\nc', empty: '', list: ['x', 'y'] },
    'Body\n',
  )
  assertEquals(text, '---\nid: ab12\ntitle: A: b c\nlist: ["x","y"]\n---\n\nBody\n')
  assertEquals(listField(parseDocument(text).fields.list), ['x', 'y'])
  assertEquals(listField('a, b'), ['a', 'b'])
  assertEquals(listField('["a, b", "c"]'), ['a, b', 'c'])
  assertEquals(formatDocument({ empty: [] }), '---\n\n---\n')
  assertEquals(parseDocument(text), {
    fields: { id: 'ab12', title: 'A: b c', list: '["x","y"]' },
    body: 'Body',
  })
  assertEquals(parseDocument('no front matter'), { fields: {}, body: 'no front matter' })
})

Deno.test('makes a slug of a title', () => {
  assertEquals(slugify('Fix the Login crash!  (again)'), 'fix-the-login-crash-again')
  assertEquals(slugify('***'), 'item')
})

Deno.test('checks the type and the size of an image', () => {
  assertEquals(imageExtension(pngOfSize(20)), 'png')
  assertEquals(imageExtension(new Uint8Array([0xff, 0xd8, 0xff, 0])), 'jpg')
  assertEquals(imageExtension(new TextEncoder().encode('<svg>')), null)
  assertEquals(checkImage(pngOfSize(150_000), { maxImageKb: 150 }), 'png')
  assertThrows(() => checkImage(pngOfSize(150_001), { maxImageKb: 150 }), /the limit is 150 kB/)
  assertThrows(() => checkImage(new Uint8Array(10), { maxImageKb: 150 }), /not a PNG/)
})

const DIFF = `diff --git a/src/app.js b/src/app.js
index 1..2 100644
--- a/src/app.js
+++ b/src/app.js
@@ -1,3 +1,3 @@ top
 one
-two
+TWO
 three
diff --git a/src/legacy/old.js b/src/legacy/new.js
similarity index 100%
rename from src/legacy/old.js
rename to src/legacy/new.js
diff --git a/my file.txt b/my file.txt
new file mode 100644
--- /dev/null
+++ b/my file.txt
@@ -0,0 +1 @@
+hello
\\ No newline at end of file
diff --git a/logo.png b/logo.png
deleted file mode 100644
Binary files a/logo.png and /dev/null differ
`

Deno.test('parses a unified diff with renames, spaces and binary files', () => {
  const files = parseDiff(DIFF)
  assertEquals(
    files.map(({ path, oldPath, status, binary, additions, deletions }) => ({
      path,
      oldPath,
      status,
      binary,
      additions,
      deletions,
    })),
    [
      {
        path: 'src/app.js',
        oldPath: 'src/app.js',
        status: 'modified',
        binary: false,
        additions: 1,
        deletions: 1,
      },
      {
        path: 'src/legacy/new.js',
        oldPath: 'src/legacy/old.js',
        status: 'renamed',
        binary: false,
        additions: 0,
        deletions: 0,
      },
      {
        path: 'my file.txt',
        oldPath: null,
        status: 'added',
        binary: false,
        additions: 1,
        deletions: 0,
      },
      {
        path: 'logo.png',
        oldPath: 'logo.png',
        status: 'deleted',
        binary: true,
        additions: 0,
        deletions: 0,
      },
    ],
  )
  assertEquals(files[0].hunks[0].lines.map((line) => [line.kind, line.old, line.new]), [
    ['context', 1, 1],
    ['del', 2, null],
    ['add', null, 2],
    ['context', 3, 3],
  ])
  assertEquals(files[2].hunks[0].lines[0].noNewline, true)
  assertEquals(lineText(files, { path: 'src/app.js', line: 2, side: 'new' }), 'TWO')
  assertEquals(lineText(files, { path: 'src/app.js', line: 2, side: 'old' }), 'two')
})

Deno.test('parses the targets of review comments', () => {
  assertEquals(parseTarget('src/a.js:42'), {
    kind: 'line',
    path: 'src/a.js',
    line: 42,
    side: 'new',
  })
  assertEquals(parseTarget('src/a.js:old:7'), {
    kind: 'line',
    path: 'src/a.js',
    line: 7,
    side: 'old',
  })
  assertEquals(parseTarget('src/a.js'), { kind: 'file', path: 'src/a.js', line: null, side: null })
  assertEquals(parseTarget('./src/'), { kind: 'folder', path: 'src/', line: null, side: null })
  assertThrows(() => parseTarget('../x'), /is not a target/)
  assertThrows(() => parseTarget(''), /is not a target/)
  assertEquals(formatTarget(parseTarget('src/a.js:old:7')), 'src/a.js:old:7')
})

Deno.test('writes a review and reads it back', () => {
  const review = {
    revision: 2,
    verdict: 'changes-requested',
    updated: null,
    summary: 'Almost there.',
    comments: [
      { id: 'c-aaaa', ...parseTarget('src/a.js:3'), quote: 'const x = 1', text: 'Name it.' },
      {
        id: 'c-bbbb',
        ...parseTarget('src/b.js'),
        quote: null,
        text: 'Rename the file.\n### not a header',
      },
      { id: 'c-cccc', ...parseTarget('src/old/'), quote: null, text: 'Move the folder.' },
    ],
  }
  const parsed = parseReview(formatReview(review), 1)
  assertEquals(parsed.revision, 2)
  assertEquals(parsed.verdict, 'changes-requested')
  assertEquals(parsed.summary, 'Almost there.')
  assertEquals(
    parsed.comments.map(({ id, target, kind, quote, text }) => ({ id, target, kind, quote, text })),
    [
      { id: 'c-aaaa', target: 'src/a.js:3', kind: 'line', quote: 'const x = 1', text: 'Name it.' },
      {
        id: 'c-bbbb',
        target: 'src/b.js',
        kind: 'file',
        quote: null,
        text: 'Rename the file.\n\\### not a header',
      },
      { id: 'c-cccc', target: 'src/old/', kind: 'folder', quote: null, text: 'Move the folder.' },
    ],
  )
})

Deno.test('builds the agent prompt and command', () => {
  assertEquals(implementPrompt(['ab12']).startsWith('Implement the todoer todo ab12.'), true)
  assertEquals(implementPrompt(['ab12', 'cd34']).includes('ab12, cd34 in one worktree'), true)
  assertEquals(agentCommandFor(['claude', '{prompt}', '-x'], 'go'), ['claude', 'go', '-x'])
  assertEquals(agentCommandFor(['codex'], 'go'), ['codex', 'go'])
})
