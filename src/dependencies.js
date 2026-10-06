import { listPrs, setPrDependsOn } from './prs.js'
import { listTodos, NotFoundError, removeTodo, updateTodo } from './todos.js'

export const REF_PATTERN = /^(todo|pr):([a-z0-9]{4,8})$/

const KIND_NAMES = { todo: 'todo', pr: 'pull request' }

export const loadGraph = async (project) => {
  const nodes = new Map()
  const todos = await listTodos(project)
  for (const todo of todos) {
    nodes.set(`todo:${todo.id}`, {
      key: `todo:${todo.id}`,
      kind: 'todo',
      id: todo.id,
      title: todo.title,
      status: todo.status,
      complete: todo.status === 'done',
      groups: todo.groups,
      dependsOn: todo.dependsOn,
      pr: todo.pr ? `pr:${todo.pr}` : null,
      todos: [],
      neededBy: [],
    })
  }
  for (const pr of await listPrs(project)) {
    const own = pr.todos.map((id) => `todo:${id}`)
    nodes.set(`pr:${pr.id}`, {
      key: `pr:${pr.id}`,
      kind: 'pr',
      id: pr.id,
      title: pr.title,
      status: pr.status,
      complete: pr.status === 'merged',
      groups: [...new Set(own.flatMap((key) => nodes.get(key)?.groups ?? []))],
      dependsOn: pr.dependsOn,
      pr: null,
      todos: own,
      neededBy: [],
    })
  }
  for (const node of nodes.values()) {
    for (const key of node.dependsOn) nodes.get(key)?.neededBy.push(node.key)
  }
  return nodes
}

export const describe = (nodes, key) => {
  const node = nodes.get(key)
  if (!node) return `${key} (missing)`
  return `${KIND_NAMES[node.kind]} ${node.id} "${node.title}" (${node.status})`
}

export const resolveRef = (nodes, text) => {
  const value = String(text ?? '').trim()
  const typed = value.match(REF_PATTERN)
  if (typed) {
    if (!nodes.has(value)) {
      throw new NotFoundError(`there is no ${KIND_NAMES[typed[1]]} ${typed[2]}`)
    }
    return value
  }
  const found = [`todo:${value}`, `pr:${value}`].filter((key) => nodes.has(key))
  if (!found.length) throw new NotFoundError(`there is no todo or pull request ${value}`)
  if (found.length > 1) {
    throw new Error(`${value} is a todo and a pull request - write todo:${value} or pr:${value}`)
  }
  return found[0]
}

export const reaches = (nodes, start, target) => {
  const seen = new Set()
  const stack = [start]
  while (stack.length) {
    const key = stack.pop()
    if (key === target) return true
    if (seen.has(key)) continue
    seen.add(key)
    stack.push(...(nodes.get(key)?.dependsOn ?? []))
  }
  return false
}

const saveDependsOn = (project, node, list) =>
  node.kind === 'todo'
    ? updateTodo(project, node.id, { dependsOn: list })
    : setPrDependsOn(project, node.id, list)

export const checkDependency = (nodes, from, to) => {
  const node = nodes.get(from)
  const target = nodes.get(to)
  if (from === to) return 'an item cannot depend on itself'
  if (node.kind === 'pr' && node.todos.includes(to)) {
    return `pull request ${node.id} implements todo ${target.id} - it cannot depend on it`
  }
  if (node.kind === 'todo' && target.kind === 'pr' && target.todos.includes(from)) {
    return `pull request ${target.id} implements todo ${node.id} - it cannot block it`
  }
  if (reaches(nodes, to, from)) {
    return `${describe(nodes, to)} already depends on ${describe(nodes, from)} - this makes a cycle`
  }
  return null
}

export const addDependency = async (project, fromText, toText) => {
  const nodes = await loadGraph(project)
  const from = resolveRef(nodes, fromText)
  const to = resolveRef(nodes, toText)
  const node = nodes.get(from)
  if (node.dependsOn.includes(to)) return { from, to, added: false }
  const problem = checkDependency(nodes, from, to)
  if (problem) throw new Error(problem)
  await saveDependsOn(project, node, [...node.dependsOn, to])
  return { from, to, added: true }
}

export const removeDependency = async (project, fromText, toText) => {
  const nodes = await loadGraph(project)
  const from = resolveRef(nodes, fromText)
  const node = nodes.get(from)
  const value = String(toText ?? '').trim()
  const to = REF_PATTERN.test(value) ? value : resolveRef(nodes, value)
  if (!node.dependsOn.includes(to)) {
    throw new NotFoundError(`${describe(nodes, from)} does not depend on ${describe(nodes, to)}`)
  }
  await saveDependsOn(project, node, node.dependsOn.filter((key) => key !== to))
  return { from, to }
}

export const removeReferences = async (project, key) => {
  const nodes = await loadGraph(project)
  for (const node of nodes.values()) {
    if (node.dependsOn.includes(key)) {
      await saveDependsOn(project, node, node.dependsOn.filter((entry) => entry !== key))
    }
  }
}

export const deleteTodo = async (project, id) => {
  const todo = await removeTodo(project, id)
  await removeReferences(project, `todo:${todo.id}`)
  return todo
}

export const effectiveDependencies = (nodes, key) => {
  const node = nodes.get(key)
  if (!node) return []
  const keys = new Set(node.dependsOn)
  for (const todoKey of node.todos) {
    for (const dependency of nodes.get(todoKey)?.dependsOn ?? []) keys.add(dependency)
  }
  return [...keys].filter((entry) => entry !== key && !node.todos.includes(entry))
}

export const blockersOf = (nodes, key) =>
  effectiveDependencies(nodes, key).filter((entry) =>
    nodes.has(entry) && !nodes.get(entry).complete
  )

export const isReady = (nodes, key) => {
  const node = nodes.get(key)
  return node.kind === 'todo' && node.status === 'todo' && blockersOf(nodes, key).length === 0
}

export const assertNotBlocked = async (project, keys, action) => {
  const nodes = await loadGraph(project)
  const problems = []
  for (const key of keys) {
    const blockers = blockersOf(nodes, key)
    if (blockers.length) {
      problems.push(
        `${describe(nodes, key)} waits for ${
          blockers.map((entry) => describe(nodes, entry)).join(', ')
        }`,
      )
    }
  }
  if (problems.length) {
    throw new Error(
      `cannot ${action}:\n  ${problems.join('\n  ')}\nfinish them first, or add --force`,
    )
  }
}

export const treeLines = (nodes, root, direction = 'dependsOn') => {
  const lines = []
  const shown = new Set()
  const walk = (key, depth, path) => {
    const children = nodes.get(key)?.[direction] ?? []
    for (const child of children) {
      const prefix = '  '.repeat(depth)
      const node = nodes.get(child)
      const mark = node?.complete ? '[x]' : '[ ]'
      if (path.has(child)) {
        lines.push(`${prefix}${mark} ${describe(nodes, child)} - cycle`)
        continue
      }
      if (shown.has(child) && (nodes.get(child)?.[direction] ?? []).length) {
        lines.push(`${prefix}${mark} ${describe(nodes, child)} - see above`)
        continue
      }
      shown.add(child)
      lines.push(`${prefix}${mark} ${describe(nodes, child)}`)
      walk(child, depth + 1, new Set([...path, child]))
    }
  }
  walk(root, 0, new Set([root]))
  return lines
}
