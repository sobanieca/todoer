import { addImage, listImages } from '../src/images.js'
import { findProject } from '../src/project.js'
import {
  addComment,
  closePr,
  listPrs,
  mergePr,
  readPrDetails,
  removeComment,
  setVerdict,
  submitPr,
} from '../src/prs.js'
import { createTodo, getTodo, listTodos, removeTodo, updateTodo } from '../src/todos.js'
import { startWork } from '../src/work.js'
import { join } from '../src/paths.js'
import { assertEquals, assertRejects } from './assert.js'
import { createRepo, pngOfSize, removeRepo, run, writeFile } from './fixtures.js'

Deno.test('keeps todos as markdown files in .todoer/todos', async () => {
  const { root, project } = await createRepo()
  try {
    const todo = await createTodo(project, { title: '  Fix the  login ', body: 'Steps' })
    assertEquals(todo.title, 'Fix the login')
    const text = await Deno.readTextFile(join(root, '.todoer/todos', todo.file))
    assertEquals(text.includes(`id: ${todo.id}\ntitle: Fix the login\nstatus: todo`), true)
    assertEquals(await Deno.readTextFile(join(root, '.todoer/.gitignore')), 'worktrees/\nruns/\n')
    await updateTodo(project, todo.id, { status: 'review', body: 'New' })
    const read = await getTodo(project, todo.id)
    assertEquals([read.status, read.body], ['review', 'New'])
    await assertRejects(() => updateTodo(project, todo.id, { status: 'later' }), /not a status/)
    await assertRejects(() => createTodo(project, { title: ' ' }), /needs a title/)
    await removeTodo(project, todo.id)
    assertEquals(await listTodos(project), [])
  } finally {
    await removeRepo(root)
  }
})

Deno.test('takes the image limit from todoer.json', async () => {
  const { root, project } = await createRepo({ maxImageKb: 2 })
  try {
    const todo = await createTodo(project, { title: 'Images' })
    const image = await addImage(project, todo.id, {
      name: 'Screen Shot.PNG',
      bytes: pngOfSize(2000),
    })
    assertEquals(image.name, '1-screen-shot.png')
    assertEquals(image.markdown, `![1-screen-shot.png](../assets/${todo.id}/1-screen-shot.png)`)
    await assertRejects(
      () => addImage(project, todo.id, { name: 'big.png', bytes: pngOfSize(2001) }),
      /the limit is 2 kB/,
    )
    assertEquals((await listImages(project, todo.id)).length, 1)
  } finally {
    await removeRepo(root)
  }
})

Deno.test('takes the default image limit of 150 kB', async () => {
  const { root, project } = await createRepo(null)
  try {
    assertEquals(project.config.maxImageKb, 150)
    const todo = await createTodo(project, { title: 'Images' })
    await addImage(project, todo.id, { name: 'ok.png', bytes: pngOfSize(150_000) })
    await assertRejects(
      () => addImage(project, todo.id, { name: 'big.png', bytes: pngOfSize(150_001) }),
      /the limit is 150 kB/,
    )
  } finally {
    await removeRepo(root)
  }
})

Deno.test('runs a todo from the worktree to the merge on main', async () => {
  const { root, project } = await createRepo()
  try {
    const first = await createTodo(project, { title: 'Upper-case two' })
    const second = await createTodo(project, { title: 'Rename old.js' })
    const work = await startWork(project, [first.id, second.id])
    assertEquals(work.branch, `todoer/${first.id}-${second.id}`)
    assertEquals(work.path, join(root, '.todoer/worktrees', `${first.id}-${second.id}`))
    const exclude = await Deno.readTextFile(join(root, '.git/info/exclude'))
    assertEquals(exclude.includes('/.todoer/worktrees/'), true)
    assertEquals((await getTodo(project, first.id)).status, 'in-progress')
    await assertRejects(() => startWork(project, [first.id]), /is in progress in/)

    const inWorktree = await findProject(work.path)
    assertEquals([inWorktree.root, inWorktree.inWorktree], [root, true])
    await assertRejects(() => submitPr(inWorktree), /no commit on/)
    await writeFile(work.path, 'src/app.js', 'one\nTWO\nthree\n')
    await assertRejects(() => submitPr(inWorktree), /not committed/)
    await run(work.path, 'mv', 'src/legacy/old.js', 'src/legacy/new.js')
    await run(work.path, 'commit', '-qam', 'Upper-case two and rename')
    await assertRejects(() => submitPr(project), /not in the main checkout/)

    const submitted = await submitPr(inWorktree, { title: 'Two fixes', message: 'Did both.' })
    const id = submitted.pr.id
    assertEquals([submitted.created, submitted.pr.revision, submitted.pr.todos], [
      true,
      1,
      [first.id, second.id],
    ])
    assertEquals((await getTodo(project, first.id)).status, 'review')
    const details = await readPrDetails(project, id)
    assertEquals(details.files.map((file) => [file.status, file.path]), [
      ['modified', 'src/app.js'],
      ['renamed', 'src/legacy/new.js'],
    ])

    const line = await addComment(project, id, { target: 'src/app.js:2', text: 'Keep it lower.' })
    assertEquals([line.target, line.quote], ['src/app.js:2', 'TWO'])
    await addComment(project, id, { target: 'src/legacy/new.js', text: 'Call it fresh.js.' })
    const folder = await addComment(project, id, { target: 'src/legacy/', text: 'Move to lib/.' })
    await removeComment(project, id, folder.id)
    await setVerdict(project, id, { verdict: 'changes-requested', summary: 'Two things.' })
    assertEquals((await getTodo(project, first.id)).status, 'in-progress')
    const review = await Deno.readTextFile(join(root, `.todoer/prs/${id}/review.v1.md`))
    assertEquals(review.includes('### c-'), true)
    assertEquals(review.includes('`src/legacy/new.js`\nCall it fresh.js.'), true)

    await run(work.path, 'mv', 'src/legacy/new.js', 'src/legacy/fresh.js')
    await run(work.path, 'commit', '-qm', 'Call it fresh.js')
    const second2 = await submitPr(inWorktree, { message: 'Renamed it.' })
    assertEquals([second2.created, second2.pr.id, second2.pr.revision], [false, id, 2])
    const latest = await readPrDetails(project, id)
    assertEquals(latest.review.comments, [])
    assertEquals(latest.previous[0].comments.length, 2)
    assertEquals(latest.pr.description, 'Did both.\n\n## Revision 2\n\nRenamed it.')
    await assertRejects(
      () => addComment(project, id, { target: 'src/app.js', text: 'x', revision: 1 }),
      /only the latest revision/,
    )

    await writeFile(root, 'README.md', 'main moved on\n')
    await run(root, 'add', 'README.md')
    await run(root, 'commit', '-qm', 'Main moves on')
    const merged = await mergePr(project, id)
    assertEquals(merged.warnings, [])
    assertEquals(merged.pr.status, 'merged')
    const log = (await run(root, 'log', '--format=%s')).stdout.trim().split('\n')
    assertEquals(log, ['Call it fresh.js', 'Upper-case two and rename', 'Main moves on', 'Start'])
    assertEquals(await Deno.readTextFile(join(root, 'src/legacy/fresh.js')), 'old\n')
    assertEquals((await getTodo(project, second.id)).status, 'done')
    const branches = (await run(root, 'branch', '--format=%(refname:short)')).stdout.trim()
    assertEquals(branches, 'main')
    assertEquals((await listPrs(project))[0].status, 'merged')
    await assertRejects(() => mergePr(project, id), /is merged/)
  } finally {
    await removeRepo(root)
  }
})

Deno.test('claims todos in a worktree that another tool made', async () => {
  const { root, project } = await createRepo()
  try {
    const todo = await createTodo(project, { title: 'Here' })
    await assertRejects(() => startWork(project, [todo.id], { here: true }), /linked worktree only/)
    const path = join(root, '.claude/worktrees/session')
    await run(root, 'worktree', 'add', '-q', '-b', 'claude/session', path, 'main')
    const result = await startWork(await findProject(path), [todo.id], { here: true })
    assertEquals([result.branch, result.path], ['claude/session', path])
    assertEquals((await getTodo(project, todo.id)).worktree, '.claude/worktrees/session')
  } finally {
    await removeRepo(root)
  }
})

Deno.test('a patch that does not apply leaves main as it was', async () => {
  const { root, project } = await createRepo()
  try {
    const todo = await createTodo(project, { title: 'Conflict' })
    const work = await startWork(project, [todo.id])
    await writeFile(work.path, 'src/app.js', 'one\nbranch\nthree\n')
    await run(work.path, 'commit', '-qam', 'Branch change')
    const { pr } = await submitPr(await findProject(work.path))
    await writeFile(root, 'src/app.js', 'one\nmain\nthree\n')
    await run(root, 'commit', '-qam', 'Main change')
    await assertRejects(() => mergePr(project, pr.id), /does not apply on main/)
    assertEquals((await run(root, 'status', '--porcelain', '--', 'src')).stdout, '')
    assertEquals((await readPrDetails(project, pr.id)).pr.status, 'open')
    const closed = await closePr(project, pr.id)
    assertEquals(closed.pr.status, 'closed')
    assertEquals((await getTodo(project, todo.id)).status, 'todo')
  } finally {
    await removeRepo(root)
  }
})
