const FENCE = '---'

export const parseDocument = (text) => {
  const normalized = text.replace(/\r\n/g, '\n')
  if (!normalized.startsWith(`${FENCE}\n`)) return { fields: {}, body: normalized }
  const end = normalized.indexOf(`\n${FENCE}`, FENCE.length)
  if (end === -1) return { fields: {}, body: normalized }
  const fields = {}
  for (const line of normalized.slice(FENCE.length + 1, end).split('\n')) {
    const colon = line.indexOf(':')
    if (colon <= 0) continue
    fields[line.slice(0, colon).trim()] = line.slice(colon + 1).trim()
  }
  const rest = normalized.slice(end + FENCE.length + 1)
  return { fields, body: rest.replace(/^\n+/, '').replace(/\s+$/, '') }
}

const oneLine = (value) => String(value).replace(/\s*\n\s*/g, ' ').trim()

export const formatDocument = (fields, body = '') => {
  const lines = Object.entries(fields)
    .filter(([, value]) => value !== undefined && value !== null && value !== '')
    .filter(([, value]) => !Array.isArray(value) || value.length > 0)
    .map(([key, value]) =>
      `${key}: ${Array.isArray(value) ? JSON.stringify(value.map(oneLine)) : oneLine(value)}`
    )
  const text = body.trim()
  return `${FENCE}\n${lines.join('\n')}\n${FENCE}\n${text ? `\n${text}\n` : ''}`
}

const parseList = (text) => {
  try {
    const value = JSON.parse(text)
    if (Array.isArray(value)) return value.map((entry) => String(entry).trim()).filter(Boolean)
  } catch {
    return null
  }
  return null
}

export const listField = (value) => {
  const text = (value ?? '').trim()
  if (text.startsWith('[')) {
    const list = parseList(text)
    if (list) return list
  }
  return text.split(',').map((entry) => entry.trim()).filter(Boolean)
}
