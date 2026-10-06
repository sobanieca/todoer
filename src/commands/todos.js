import { requireOnlyFlags } from '../args.js'
import { blockersOf, deleteTodo, describe, isReady, loadGraph } from '../dependencies.js'
import { formatKb, listImages } from '../images.js'
import { logger } from '../logger.js'
import { absoluteFromRoot, findProject } from '../project.js'
import {
  createTodo,
  getTodo,
  listTodos,
  OPEN_STATUSES,
  STATUSES,
  todoFilePath,
  updateTodo,
} from '../todos.js'

const printJson = (value) => console.log(JSON.stringify(value, null, 2))

const statusFilter = (flags) => {
  if (flags.status) {
    const wanted = flags.status.split(',').map((status) => status.trim())
    for (const status of wanted) {
      if (!STATUSES.includes(status)) {
        throw new Error(`"${status}" is not a status - use ${STATUSES.join(', ')}`)
      }
    }
    return wanted
  }
  return flags.all ? STATUSES : OPEN_STATUSES
}

export const ls = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['status', 'all', 'group', 'ready', 'json'], 'ls')
  if (positional.length > 1) {
    throw new Error('use: todoer ls [--status <list>] [--all] [--group <name>] [--ready] [--json]')
  }
  const project = await findProject()
  const statuses = statusFilter(flags)
  const nodes = await loadGraph(project)
  const todos = (await listTodos(project))
    .filter((todo) => statuses.includes(todo.status))
    .filter((todo) => !flags.group || flags.group.every((name) => todo.groups.includes(name)))
    .filter((todo) => !flags.ready || isReady(nodes, `todo:${todo.id}`))
    .map(({ body: _body, ...todo }) => ({
      ...todo,
      blockedBy: blockersOf(nodes, `todo:${todo.id}`),
    }))
  if (flags.json) return printJson(todos)
  if (!todos.length) return logger.plain('No todos match. Add one with: todoer add "<title>"')
  for (const todo of todos) {
    const extra = [
      todo.pr ? `pr ${todo.pr}` : null,
      todo.blockedBy.length
        ? `waits for ${todo.blockedBy.map((key) => key.split(':')[1]).join(', ')}`
        : null,
      todo.groups.length ? `[${todo.groups.join('] [')}]` : null,
    ].filter(Boolean).join('  ')
    logger.plain(`${todo.id}  ${todo.status.padEnd(11)}  ${todo.title}${extra ? `  ${extra}` : ''}`)
  }
}

export const add = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['body', 'group', 'json'], 'add')
  const title = positional.slice(1).join(' ')
  if (!title) throw new Error('use: todoer add "<title>" [--body "<description>"] [--group <name>]')
  const project = await findProject()
  const todo = await createTodo(project, {
    title,
    body: flags.body ?? '',
    groups: flags.group ?? [],
  })
  if (flags.json) return printJson(todo)
  logger.plain(`Added todo ${todo.id}: ${todo.title}`)
}

export const show = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['json'], 'show')
  const [, id] = positional
  if (!id || positional.length > 2) throw new Error('use: todoer show <id> [--json]')
  const project = await findProject()
  const todo = await getTodo(project, id)
  const images = await listImages(project, todo.id)
  const file = todoFilePath(project, todo)
  const worktree = todo.worktree ? absoluteFromRoot(project, todo.worktree) : null
  const nodes = await loadGraph(project)
  const key = `todo:${todo.id}`
  const blockedBy = blockersOf(nodes, key)
  const neededBy = nodes.get(key).neededBy
  if (flags.json) return printJson({ ...todo, file, worktree, images, blockedBy, neededBy })
  const lines = [
    `${todo.id}  ${todo.title}`,
    `status: ${todo.status}`,
    todo.branch ? `branch: ${todo.branch}` : null,
    worktree ? `worktree: ${worktree}` : null,
    todo.pr ? `pull request: ${todo.pr}` : null,
    todo.groups.length ? `groups: ${todo.groups.join(', ')}` : null,
    ...todo.dependsOn.map((entry) =>
      `depends on: ${describe(nodes, entry)}${nodes.get(entry)?.complete ? '' : ' - not done'}`
    ),
    blockedBy.length
      ? `BLOCKED: wait for ${blockedBy.map((entry) => describe(nodes, entry)).join(', ')}`
      : null,
    ...neededBy.map((entry) => `needed by: ${describe(nodes, entry)}`),
    `file: ${file}`,
    '',
    todo.body.trim() || '(no description)',
  ]
  if (images.length) {
    lines.push('', 'Images:')
    for (const image of images) lines.push(`  ${image.path}  (${formatKb(image.size)})`)
  }
  logger.plain(lines.filter((line) => line !== null).join('\n'))
}

export const status = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'status')
  const [, id, value] = positional
  if (!id || !value || positional.length > 3) {
    throw new Error(`use: todoer status <id> <${STATUSES.join('|')}>`)
  }
  const project = await findProject()
  const todo = await updateTodo(project, id, { status: value })
  logger.plain(`Todo ${todo.id} is ${todo.status}`)
}

export const rm = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'rm')
  const [, id] = positional
  if (!id || positional.length > 2) throw new Error('use: todoer rm <id>')
  const project = await findProject()
  const todo = await deleteTodo(project, id)
  logger.plain(`Removed todo ${todo.id}: ${todo.title}`)
}
