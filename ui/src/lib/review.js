export const fileTarget = (path) => path

export const folderTarget = (path) => (path.endsWith('/') ? path : `${path}/`)

export const lineTarget = (path, line, side) =>
  side === 'old' ? `${path}:old:${line}` : `${path}:${line}`

const filePaths = (file) => [file.path, file.oldPath].filter(Boolean)

export const commentsOfFile = (comments, file) => {
  const paths = filePaths(file)
  return comments.filter((comment) => comment.kind !== 'folder' && paths.includes(comment.path))
}

export const commentsOfLine = (comments, line) =>
  comments.filter((comment) =>
    comment.kind === 'line' &&
    (comment.side === 'old'
      ? line.kind !== 'add' && line.old === comment.line
      : line.kind !== 'del' && line.new === comment.line)
  )

export const commentsOfFolder = (comments, folder) =>
  comments.filter((comment) => comment.kind === 'folder' && comment.path === folderTarget(folder))

export const folderCounts = (comments) => {
  const counts = {}
  for (const comment of comments) {
    if (comment.kind === 'folder') counts[comment.path] = (counts[comment.path] ?? 0) + 1
  }
  return counts
}

const sortNodes = (nodes) =>
  nodes
    .sort((a, b) => {
      if (Boolean(a.children) !== Boolean(b.children)) return a.children ? -1 : 1
      return a.name.localeCompare(b.name)
    })
    .map((node) => (node.children ? { ...node, children: sortNodes(node.children) } : node))

export const fileTree = (files) => {
  const root = []
  for (const file of files) {
    const parts = file.path.split('/')
    let level = root
    let prefix = ''
    for (const part of parts.slice(0, -1)) {
      prefix = `${prefix}${part}/`
      let folder = level.find((node) => node.children && node.path === prefix)
      if (!folder) {
        folder = { kind: 'folder', name: part, path: prefix, children: [] }
        level.push(folder)
      }
      level = folder.children
    }
    level.push({ kind: 'file', name: parts.at(-1), path: file.path, file })
  }
  return sortNodes(root)
}

const fileLabel = (node, comments) => {
  const { file } = node
  const count = commentsOfFile(comments, file).length
  const change = file.binary ? 'binary' : `+${file.additions} -${file.deletions}`
  const status = file.status === 'modified' ? '' : ` ${file.status}`
  return `${node.name}  ${change}${status}${count ? `  (${count})` : ''}`
}

export const treeItems = (files, comments) => {
  const counts = folderCounts(comments)
  const toItem = (node) =>
    node.children
      ? {
        id: `dir:${node.path}`,
        label: `${node.name}/${counts[node.path] ? `  (${counts[node.path]})` : ''}`,
        children: node.children.map(toItem),
      }
      : { id: `file:${node.path}`, label: fileLabel(node, comments) }
  return fileTree(files).map(toItem)
}

export const folderIds = (items) =>
  items.flatMap((item) => (item.children ? [item.id, ...folderIds(item.children)] : []))
