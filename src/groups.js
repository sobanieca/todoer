import { blockersOf, isReady, loadGraph } from './dependencies.js'
import { cleanGroupName, getTodo, listTodos, NotFoundError, updateTodo } from './todos.js'

export const summarizeGroups = (nodes) => {
  const groups = new Map()
  for (const node of nodes.values()) {
    if (node.kind !== 'todo') continue
    for (const name of node.groups) {
      if (!groups.has(name)) {
        groups.set(name, {
          name,
          total: 0,
          todo: 0,
          inProgress: 0,
          review: 0,
          done: 0,
          ready: 0,
          blocked: 0,
          prs: new Set(),
          openPrs: new Set(),
        })
      }
      const group = groups.get(name)
      group.total++
      if (node.status === 'todo') group.todo++
      if (node.status === 'in-progress') group.inProgress++
      if (node.status === 'review') group.review++
      if (node.status === 'done') group.done++
      if (isReady(nodes, node.key)) group.ready++
      else if (!node.complete && blockersOf(nodes, node.key).length) group.blocked++
      if (node.pr) {
        group.prs.add(node.pr)
        if (nodes.get(node.pr)?.status === 'open') group.openPrs.add(node.pr)
      }
    }
  }
  return [...groups.values()]
    .map((group) => ({
      ...group,
      prs: group.prs.size,
      openPrs: group.openPrs.size,
      percent: Math.round((group.done / group.total) * 100),
    }))
    .sort((a, b) => a.name.localeCompare(b.name))
}

export const listGroups = async (project) => summarizeGroups(await loadGraph(project))

const changeGroups = async (project, change) => {
  let count = 0
  for (const todo of await listTodos(project)) {
    const next = change(todo.groups)
    if (
      next.length !== todo.groups.length || next.some((name, index) => name !== todo.groups[index])
    ) {
      await updateTodo(project, todo.id, { groups: next })
      count++
    }
  }
  return count
}

export const renameGroup = async (project, from, to) => {
  const oldName = cleanGroupName(from)
  const newName = cleanGroupName(to)
  const count = await changeGroups(
    project,
    (groups) =>
      groups.includes(oldName)
        ? [...new Set(groups.map((name) => (name === oldName ? newName : name)))]
        : groups,
  )
  if (!count) throw new NotFoundError(`there is no group "${oldName}"`)
  return count
}

export const removeGroup = async (project, name) => {
  const clean = cleanGroupName(name)
  const count = await changeGroups(project, (groups) => groups.filter((entry) => entry !== clean))
  if (!count) throw new NotFoundError(`there is no group "${clean}"`)
  return count
}

export const addToGroup = async (project, name, ids) => {
  const clean = cleanGroupName(name)
  if (!ids.length) throw new Error('name one or more todos')
  const todos = []
  for (const id of ids) todos.push(await getTodo(project, id))
  for (const todo of todos) {
    if (!todo.groups.includes(clean)) {
      await updateTodo(project, todo.id, { groups: [...todo.groups, clean] })
    }
  }
  return todos.length
}

export const removeFromGroup = async (project, name, ids) => {
  const clean = cleanGroupName(name)
  for (const id of ids) {
    const todo = await getTodo(project, id)
    await updateTodo(project, todo.id, { groups: todo.groups.filter((entry) => entry !== clean) })
  }
  return ids.length
}
