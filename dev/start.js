import { parseArgs, parsePort } from '../src/args.js'
import { git } from '../src/git.js'
import { addDependency } from '../src/dependencies.js'
import { addImage } from '../src/images.js'
import { findProject } from '../src/project.js'
import { addComment, submitPr } from '../src/prs.js'
import { createTodo, updateTodo } from '../src/todos.js'
import { startUi } from '../src/ui-server.js'
import { startWork } from '../src/work.js'
import { join } from '../src/paths.js'

const PNG = Uint8Array.from(
  atob(
    'iVBORw0KGgoAAAANSUhEUgAAAGAAAAAwCAIAAABhdOiYAAAAfklEQVR42u3ZMQrAIBAAQX/gT1PmwVYprYWAhZAmYKxEzMCWcnDTHYZ0ZXUKCAABAgQIECBAgF7F82jlu3z2PO608hxAgOYDbbP8yBxAgAABAgQIECBAgAABWgTIsQoIECBAgAABAgQIECBA/sVc84AA/QtIgAABAgQIEKCdq3V4bFE1VWO3AAAAAElFTkSuQmCC',
  ),
  (char) => char.charCodeAt(0),
)

const write = async (root, path, text) => {
  await Deno.mkdir(join(root, path.slice(0, path.lastIndexOf('/') + 1) || '.'), { recursive: true })
  await Deno.writeTextFile(join(root, path), text)
}

const run = (root, ...args) => git(args, { cwd: root })

const seed = async (root) => {
  await run(root, 'init', '-q', '-b', 'main')
  await run(root, 'config', 'user.email', 'demo@example.com')
  await run(root, 'config', 'user.name', 'Demo')
  await write(
    root,
    'src/cart.js',
    [
      'export const total = (items) => {',
      '  let sum = 0',
      '  for (const item of items) sum += item.price',
      '  return sum',
      '}',
      '',
    ].join('\n'),
  )
  await write(root, 'src/legacy/format.js', 'export const money = (value) => `$${value}`\n')
  await write(root, 'README.md', '# Shop\n\nA small demo shop.\n')
  await run(root, 'add', '-A')
  await run(root, 'commit', '-qm', 'Start the shop')
  const project = await findProject(root)
  const quantity = await createTodo(project, {
    title: 'Count the quantity in the cart total',
    groups: ['tests refactoring'],
    body: 'The total ignores `item.quantity`. A cart with 2 books shows the price of 1.\n\n' +
      'Add a test for it.',
  })
  const image = await addImage(project, quantity.id, { name: 'cart.png', bytes: PNG })
  await updateTodo(project, quantity.id, { body: `${quantity.body}\n\n${image.markdown}` })
  const prices = await createTodo(project, {
    title: 'Show prices with two decimals',
    groups: ['money format'],
  })
  const theme = await createTodo(project, {
    title: 'Add a dark theme to the shop',
    body: 'Follow the system setting.',
  })
  await addDependency(project, theme.id, prices.id)
  const steps = [
    'Add a test helper for the cart',
    'Move the cart tests to test/cart/',
    'Split the cart tests by feature',
    'Add fixtures for the products',
    'Use the fixtures in the cart tests',
    'Remove the old test data',
    'Run the tests in parallel',
    'Add a coverage report',
    'Fail the build under 80% coverage',
    'Write the testing guide',
  ]
  let previous = null
  for (const title of steps) {
    const todo = await createTodo(project, { title, groups: ['tests refactoring'] })
    if (previous) await addDependency(project, todo.id, previous.id)
    previous = todo
  }
  const bump = await createTodo(project, {
    title: 'Bump some-npm-package to 0.3.5',
    groups: ['bump some-npm-package version to 0.3.5', 'tests refactoring'],
  })
  const lockfile = await createTodo(project, {
    title: 'Update the lock file',
    groups: ['bump some-npm-package version to 0.3.5'],
  })
  await addDependency(project, lockfile.id, bump.id)
  await addDependency(project, previous.id, bump.id)
  const { path } = await startWork(project, [quantity.id])
  await write(
    path,
    'src/cart.js',
    [
      'export const total = (items) => {',
      '  let sum = 0',
      '  for (const item of items) sum += item.price * (item.quantity ?? 1)',
      '  return sum',
      '}',
      '',
    ].join('\n'),
  )
  await write(
    path,
    'test/cart.test.js',
    [
      "import { total } from '../src/cart.js'",
      '',
      "Deno.test('counts the quantity', () => {",
      '  if (total([{ price: 2, quantity: 3 }]) !== 6) throw new Error()',
      '})',
      '',
    ].join('\n'),
  )
  await run(path, 'mv', 'src/legacy/format.js', 'src/legacy/money.js')
  await run(path, 'add', '-A')
  await run(path, 'commit', '-qm', 'Count the quantity in the cart total')
  const { pr } = await submitPr(await findProject(path), {
    message: 'The total multiplies each price by the quantity. A new test covers it.',
  })
  await addDependency(project, bump.id, `pr:${pr.id}`)
  await addComment(project, pr.id, {
    target: 'src/cart.js:3',
    text: 'Use `reduce` here.',
  })
  await addComment(project, pr.id, {
    target: 'src/legacy/',
    text: 'Move this folder to `src/format/`.',
  })
}

const { flags } = parseArgs(Deno.args)
const root = await Deno.makeTempDir({ prefix: 'todoer-demo-' })
await seed(root)
const project = await findProject(root)
const server = await startUi(project, { port: flags.port ? parsePort(flags.port) : 8020 })
console.log(`Demo project: ${root}`)
console.log(`Open ${server.url}  (Ctrl+C stops it and removes the demo project)`)
const stop = async () => {
  await server.shutdown()
  await Deno.remove(root, { recursive: true })
  Deno.exit(0)
}
Deno.addSignalListener('SIGINT', stop)
Deno.addSignalListener('SIGTERM', stop)
