export const messageOf = (error) => error?.body?.error ?? error?.message ?? String(error)

export const assetUrl = (todoId, name) =>
  `/files/assets/${encodeURIComponent(todoId)}/${encodeURIComponent(name)}`

export const previewMarkdown = (text) =>
  (text ?? '').replace(
    /\]\(\.\.\/assets\/([a-z0-9]+)\/([a-z0-9.-]+)\)/g,
    (_, todoId, name) => `](${assetUrl(todoId, name)})`,
  )

export const formatKb = (bytes) => `${Math.round(bytes / 100) / 10} kB`

export const formatTime = (value) => {
  if (!value) return '-'
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

export const appendImage = (body, markdown) => {
  const text = (body ?? '').replace(/\s+$/, '')
  return text ? `${text}\n\n${markdown}` : markdown
}

export const imageFromClipboard = (event) => {
  for (const item of event.clipboardData?.items ?? []) {
    if (item.kind === 'file' && item.type.startsWith('image/')) return item.getAsFile()
  }
  return null
}

export const uploadImage = (api, todoId, file) =>
  api.post(`/todos/${todoId}/images`, {
    query: { name: file.name || 'pasted' },
    body: file,
    headers: { 'content-type': file.type || 'application/octet-stream' },
  })

export const STATUS_LABELS = {
  'todo': 'To do',
  'in-progress': 'In progress',
  'review': 'In review',
  'done': 'Done',
}

export const STATUS_COLORS = {
  'todo': 'gray',
  'in-progress': 'blue',
  'review': 'amber',
  'done': 'green',
}

export const VERDICT_LABELS = {
  'pending': 'Not reviewed',
  'changes-requested': 'Changes requested',
  'approved': 'Approved',
}

export const VERDICT_COLORS = {
  'pending': 'gray',
  'changes-requested': 'orange',
  'approved': 'green',
}

export const PR_COLORS = { open: 'blue', merged: 'violet', closed: 'gray' }
