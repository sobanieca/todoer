import { formatDocument, listField, parseDocument } from './frontmatter.js'
import { listDirectory, removeIfExists, writeFileAtomic } from './fs-utils.js'
import { ID_PATTERN, now, slugify, uniqueId } from './ids.js'
import { ensureDataDir, projectPath } from './project.js'
import { join } from './paths.js'

export const STATUSES = ['todo', 'in-progress', 'review', 'done']

export const OPEN_STATUSES = ['todo', 'in-progress', 'review']

const FIELDS = [
  'id',
  'title',
  'status',
  'groups',
  'dependsOn',
  'created',
  'updated',
  'branch',
  'worktree',
  'pr',
]

export const cleanGroupName = (name) => {
  const text = String(name ?? '').replace(/\s+/g, ' ').trim()
  if (!text) throw new Error('a group needs a name')
  if (text.length > 100) throw new Error('a group name has 100 characters or fewer')
  return text
}

export const cleanGroups = (groups) => {
  if (!Array.isArray(groups)) throw new Error('"groups" is a list of group names')
  return [...new Set(groups.map(cleanGroupName))]
}

export class NotFoundError extends Error {}

const todosDir = (project) => projectPath(project, 'todos')

const toTodo = (file, text) => {
  const { fields, body } = parseDocument(text)
  const id = fields.id ?? file.split('-')[0].replace(/\.md$/, '')
  return {
    id,
    title: fields.title ?? id,
    status: STATUSES.includes(fields.status) ? fields.status : 'todo',
    groups: listField(fields.groups),
    dependsOn: listField(fields.dependsOn),
    created: fields.created ?? null,
    updated: fields.updated ?? fields.created ?? null,
    branch: fields.branch || null,
    worktree: fields.worktree || null,
    pr: fields.pr || null,
    file,
    body,
  }
}

const serialize = (todo) =>
  formatDocument(Object.fromEntries(FIELDS.map((key) => [key, todo[key]])), todo.body)

export const listTodos = async (project) => {
  const todos = []
  for (const entry of await listDirectory(todosDir(project))) {
    if (!entry.isFile || !entry.name.endsWith('.md')) continue
    const text = await Deno.readTextFile(join(todosDir(project), entry.name))
    todos.push(toTodo(entry.name, text))
  }
  return todos.sort((a, b) => (b.created ?? '').localeCompare(a.created ?? ''))
}

const findFile = async (project, id) => {
  if (!ID_PATTERN.test(id ?? '')) throw new NotFoundError(`"${id}" is not a todo id`)
  for (const entry of await listDirectory(todosDir(project))) {
    if (entry.name === `${id}.md` || entry.name.startsWith(`${id}-`)) return entry.name
  }
  throw new NotFoundError(`there is no todo ${id} - run todoer ls`)
}

export const getTodo = async (project, id) => {
  const file = await findFile(project, id)
  return toTodo(file, await Deno.readTextFile(join(todosDir(project), file)))
}

const cleanTitle = (title) => {
  const text = String(title ?? '').replace(/\s+/g, ' ').trim()
  if (!text) throw new Error('a todo needs a title')
  if (text.length > 200) throw new Error('a todo title has 200 characters or fewer')
  return text
}

export const takenIds = async (project) =>
  new Set([
    ...(await listTodos(project)).map((todo) => todo.id),
    ...(await listDirectory(projectPath(project, 'prs'))).map((entry) => entry.name),
  ])

export const createTodo = async (project, { title, body = '', groups = [] }) => {
  await ensureDataDir(project)
  const clean = cleanTitle(title)
  const cleanedGroups = cleanGroups(groups)
  const taken = await takenIds(project)
  const id = uniqueId(taken)
  const stamp = now()
  const todo = {
    id,
    title: clean,
    status: 'todo',
    groups: cleanedGroups,
    dependsOn: [],
    created: stamp,
    updated: stamp,
    branch: null,
    worktree: null,
    pr: null,
    file: `${id}-${slugify(clean)}.md`,
    body: String(body ?? ''),
  }
  await writeFileAtomic(join(todosDir(project), todo.file), serialize(todo))
  return todo
}

export const updateTodo = async (project, id, changes) => {
  const todo = await getTodo(project, id)
  const next = { ...todo }
  if (changes.title !== undefined) next.title = cleanTitle(changes.title)
  if (changes.body !== undefined) next.body = String(changes.body ?? '')
  if (changes.status !== undefined) {
    if (!STATUSES.includes(changes.status)) {
      throw new Error(`"${changes.status}" is not a status - use ${STATUSES.join(', ')}`)
    }
    next.status = changes.status
  }
  if (changes.groups !== undefined) next.groups = cleanGroups(changes.groups)
  if (changes.dependsOn !== undefined) next.dependsOn = [...new Set(changes.dependsOn)]
  for (const key of ['branch', 'worktree', 'pr']) {
    if (changes[key] !== undefined) next[key] = changes[key] || null
  }
  next.updated = now()
  await writeFileAtomic(join(todosDir(project), todo.file), serialize(next))
  return next
}

export const removeTodo = async (project, id) => {
  const todo = await getTodo(project, id)
  await removeIfExists(join(todosDir(project), todo.file))
  await removeIfExists(projectPath(project, 'assets', todo.id), { recursive: true })
  return todo
}

export const todoFilePath = (project, todo) => join(todosDir(project), todo.file)
