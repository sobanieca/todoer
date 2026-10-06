import {
  addDependency,
  assertNotBlocked,
  blockersOf,
  deleteTodo,
  isReady,
  loadGraph,
  removeDependency,
  treeLines,
} from '../src/dependencies.js'
import { addToGroup, listGroups, removeGroup, renameGroup } from '../src/groups.js'
import { findProject } from '../src/project.js'
import { mergePr, submitPr } from '../src/prs.js'
import { createTodo, getTodo, updateTodo } from '../src/todos.js'
import { startWork } from '../src/work.js'
import { assertEquals, assertRejects } from './assert.js'
import { createRepo, removeRepo, run, writeFile } from './fixtures.js'

Deno.test('groups todos and counts the progress of each group', async () => {
  const { root, project } = await createRepo()
  try {
    const a = await createTodo(project, { title: 'A', groups: ['tests refactoring'] })
    const b = await createTodo(project, {
      title: 'B',
      groups: ['tests refactoring', 'bump some-npm-package version to 0.3.5'],
    })
    await createTodo(project, { title: 'C' })
    await updateTodo(project, a.id, { status: 'done' })
    await assertRejects(() => createTodo(project, { title: 'D', groups: [' '] }), /needs a name/)
    let groups = await listGroups(project)
    assertEquals(groups.map(({ name, total, done, percent }) => ({ name, total, done, percent })), [
      { name: 'bump some-npm-package version to 0.3.5', total: 1, done: 0, percent: 0 },
      { name: 'tests refactoring', total: 2, done: 1, percent: 50 },
    ])
    await renameGroup(project, 'tests refactoring', 'tests')
    await addToGroup(project, 'tests', [b.id])
    assertEquals((await getTodo(project, b.id)).groups, [
      'tests',
      'bump some-npm-package version to 0.3.5',
    ])
    await removeGroup(project, 'bump some-npm-package version to 0.3.5')
    groups = await listGroups(project)
    assertEquals(groups.map((group) => group.name), ['tests'])
    await assertRejects(() => renameGroup(project, 'nope', 'x'), /no group "nope"/)
  } finally {
    await removeRepo(root)
  }
})

Deno.test('links todos and pull requests in any depth and refuses a cycle', async () => {
  const { root, project } = await createRepo()
  try {
    const chain = []
    for (let index = 0; index < 30; index++) {
      chain.push(await createTodo(project, { title: `Step ${index}` }))
      if (index > 0) await addDependency(project, chain[index].id, chain[index - 1].id)
    }
    const nodes = await loadGraph(project)
    const last = `todo:${chain[29].id}`
    assertEquals(treeLines(nodes, last).length, 29)
    assertEquals(treeLines(nodes, `todo:${chain[0].id}`, 'neededBy').length, 29)
    assertEquals(blockersOf(nodes, last), [`todo:${chain[28].id}`])
    assertEquals(isReady(nodes, `todo:${chain[0].id}`), true)
    await assertRejects(() => addDependency(project, chain[0].id, chain[29].id), /makes a cycle/)
    await assertRejects(() => addDependency(project, chain[0].id, chain[0].id), /itself/)
    assertEquals((await addDependency(project, chain[2].id, chain[1].id)).added, false)
    await removeDependency(project, chain[2].id, chain[1].id)
    assertEquals((await getTodo(project, chain[2].id)).dependsOn, [])
    await deleteTodo(project, chain[5].id)
    assertEquals((await getTodo(project, chain[6].id)).dependsOn, [])
  } finally {
    await removeRepo(root)
  }
})

Deno.test('shows a node that two paths reach once, and marks the second path', async () => {
  const { root, project } = await createRepo()
  try {
    const base = await createTodo(project, { title: 'Base' })
    const left = await createTodo(project, { title: 'Left' })
    const right = await createTodo(project, { title: 'Right' })
    const top = await createTodo(project, { title: 'Top' })
    const deep = await createTodo(project, { title: 'Deep' })
    await addDependency(project, base.id, deep.id)
    for (const side of [left, right]) await addDependency(project, side.id, base.id)
    await addDependency(project, top.id, left.id)
    await addDependency(project, top.id, right.id)
    const lines = treeLines(await loadGraph(project), `todo:${top.id}`)
    assertEquals(lines.filter((line) => line.includes('"Base"')).length, 2)
    assertEquals(lines.filter((line) => line.includes('see above')).length, 1)
    assertEquals(lines.filter((line) => line.includes('"Deep"')).length, 1)
  } finally {
    await removeRepo(root)
  }
})

Deno.test('blocks the start of a todo and the merge of a pull request', async () => {
  const { root, project } = await createRepo()
  try {
    const first = await createTodo(project, { title: 'First' })
    const second = await createTodo(project, { title: 'Second' })
    await addDependency(project, second.id, first.id)
    await assertRejects(() => startWork(project, [second.id]), /waits for todo/)
    const work = await startWork(project, [first.id])
    await writeFile(work.path, 'src/app.js', 'changed\n')
    await run(work.path, 'commit', '-qam', 'First')
    const { pr } = await submitPr(await findProject(work.path))
    const other = await createTodo(project, { title: 'Other' })
    await addDependency(project, `pr:${pr.id}`, other.id)
    await assertRejects(() => assertNotBlocked(project, [`pr:${pr.id}`], 'merge'), /waits for/)
    await assertRejects(
      () => addDependency(project, `pr:${pr.id}`, first.id),
      /implements todo/,
    )
    await updateTodo(project, other.id, { status: 'done' })
    await assertNotBlocked(project, [`pr:${pr.id}`], 'merge')
    await mergePr(project, pr.id)
    const nodes = await loadGraph(project)
    assertEquals(blockersOf(nodes, `todo:${second.id}`), [])
    assertEquals(isReady(nodes, `todo:${second.id}`), true)
  } finally {
    await removeRepo(root)
  }
})
