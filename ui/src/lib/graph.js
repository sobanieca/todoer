const KIND_LABELS = { todo: 'Todo', pr: 'PR' }

const STATUS_WORDS = {
  'todo': 'to do',
  'in-progress': 'in progress',
  'review': 'in review',
  'done': 'done',
  'open': 'open',
  'merged': 'merged',
  'closed': 'closed',
}

export const nodeMap = (nodes) => new Map((nodes ?? []).map((node) => [node.key, node]))

export const refName = (key) => {
  const [kind, id] = key.split(':')
  return `${KIND_LABELS[kind] ?? kind} ${id}`
}

export const nodeLabel = (node) => {
  const waits = !node.complete && node.blockedBy?.length
    ? `, waits for ${node.blockedBy.length}`
    : ''
  return `${refName(node.key)} - ${node.title} (${
    STATUS_WORDS[node.status] ?? node.status
  }${waits})`
}

export const nodeHref = (node) =>
  node.kind === 'pr'
    ? `/prs/${encodeURIComponent(node.id)}`
    : `/todos/${encodeURIComponent(node.id)}`

const SEPARATOR = '>'

export const pathDepth = (path) => path.split(SEPARATOR).length - 1

export const keyOfPath = (path) => path.slice(path.lastIndexOf(SEPARATOR) + 1)

export const dependencyTree = (map, { root = null, roots = [], direction, expanded, iconOf }) => {
  const open = expanded === 'all' ? null : new Set(expanded ?? [])
  const shown = new Set()
  const item = (key, path, ancestors) => {
    const node = map.get(key)
    if (!node) return { id: path, label: `${refName(key)} - missing` }
    const base = { id: path, label: nodeLabel(node) }
    const icon = iconOf?.(node)
    if (icon) base.icon = icon
    const children = node[direction] ?? []
    if (ancestors.has(key)) return { ...base, label: `${base.label} - cycle` }
    if (!children.length) return base
    if (shown.has(key)) return { ...base, label: `${base.label} - see above` }
    shown.add(key)
    if (open && !open.has(path)) {
      return { ...base, children: [{ id: `${path}${SEPARATOR}...`, label: 'More' }] }
    }
    const next = new Set(ancestors).add(key)
    return {
      ...base,
      children: children.map((child) => item(child, `${path}${SEPARATOR}${child}`, next)),
    }
  }
  if (root) {
    const node = map.get(root)
    const ancestors = new Set([root])
    return (node?.[direction] ?? []).map((child) =>
      item(child, `${root}${SEPARATOR}${child}`, ancestors)
    )
  }
  return roots.map((key) => item(key, key, new Set()))
}

export const parentIds = (items) =>
  items.flatMap((item) => (item.children ? [item.id, ...parentIds(item.children)] : []))

export const itemIds = (items) =>
  items.flatMap((item) => [item.id, ...(item.children ? itemIds(item.children) : [])])

export const countItems = (items) =>
  items.reduce((sum, item) => sum + 1 + (item.children ? countItems(item.children) : 0), 0)

const reaches = (map, start, target) => {
  const seen = new Set()
  const stack = [start]
  while (stack.length) {
    const key = stack.pop()
    if (key === target) return true
    if (seen.has(key)) continue
    seen.add(key)
    stack.push(...(map.get(key)?.dependsOn ?? []))
  }
  return false
}

export const canDependOn = (map, from, to) => {
  const node = map.get(from)
  const target = map.get(to)
  if (!node || !target || from === to) return false
  if (node.dependsOn.includes(to)) return false
  if (node.todos?.includes(to) || target.todos?.includes(from)) return false
  return !reaches(map, to, from)
}

export const dependencyOptions = (map, from) =>
  [...map.values()]
    .filter((node) => canDependOn(map, from, node.key))
    .sort((a, b) => Number(a.complete) - Number(b.complete) || a.key.localeCompare(b.key))
    .map((node) => ({ value: node.key, label: nodeLabel(node) }))

export const groupSlug = (name) =>
  name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '') || 'group'

export const groupNames = (nodes) =>
  [...new Set((nodes ?? []).flatMap((node) => (node.kind === 'todo' ? node.groups : [])))].sort()

export const groupRoots = (map, keys) => {
  const inside = new Set(keys)
  const needed = new Set()
  for (const key of keys) {
    for (const dependency of map.get(key)?.dependsOn ?? []) {
      if (inside.has(dependency)) needed.add(dependency)
    }
  }
  const roots = keys.filter((key) => !needed.has(key))
  return roots.length ? roots : keys
}
