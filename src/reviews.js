import { formatDocument, parseDocument } from './frontmatter.js'
import { readTextIfExists, writeFileAtomic } from './fs-utils.js'
import { now, randomId } from './ids.js'

export const VERDICTS = ['pending', 'changes-requested', 'approved']

const HEADER = /^### (c-[a-z0-9]+) `(.*)`$/

export const parseTarget = (target) => {
  const value = String(target ?? '').trim().replace(/^\.\//, '')
  if (!value || value.startsWith('/') || value.split('/').includes('..')) {
    throw new Error(`"${target}" is not a target - use path:line, a file path or a folder path/`)
  }
  if (value.endsWith('/')) return { kind: 'folder', path: value, line: null, side: null }
  const old = value.match(/^(.+):old:(\d+)$/)
  if (old) return { kind: 'line', path: old[1], line: Number(old[2]), side: 'old' }
  const line = value.match(/^(.+):(\d+)$/)
  if (line) return { kind: 'line', path: line[1], line: Number(line[2]), side: 'new' }
  return { kind: 'file', path: value, line: null, side: null }
}

export const formatTarget = ({ kind, path, line, side }) => {
  if (kind === 'line') return side === 'old' ? `${path}:old:${line}` : `${path}:${line}`
  if (kind === 'folder') return path.endsWith('/') ? path : `${path}/`
  return path
}

const escapeText = (text) => text.trim().replace(/^### /gm, '\\### ')

export const parseReview = (text, revision) => {
  const { fields, body } = parseDocument(text ?? '')
  const lines = body.split('\n')
  const summary = []
  const comments = []
  let current = null
  for (const line of lines) {
    const match = line.match(HEADER)
    if (match) {
      current = { id: match[1], ...parseTarget(match[2]), quote: null, lines: [] }
      current.target = formatTarget(current)
      comments.push(current)
      continue
    }
    if (!current) summary.push(line)
    else if (current.kind === 'line' && current.lines.length === 0 && line.startsWith('> ')) {
      current.quote = line.slice(2)
    } else current.lines.push(line)
  }
  return {
    revision: Number(fields.revision ?? revision),
    verdict: VERDICTS.includes(fields.verdict) ? fields.verdict : 'pending',
    updated: fields.updated ?? null,
    summary: summary.join('\n').trim(),
    comments: comments.map(({ lines: textLines, ...comment }) => ({
      ...comment,
      text: textLines.join('\n').trim(),
    })),
  }
}

export const formatReview = (review) => {
  const parts = [review.summary.trim()]
  for (const comment of review.comments) {
    const block = [`### ${comment.id} \`${formatTarget(comment)}\``]
    if (comment.kind === 'line' && comment.quote !== null && comment.quote !== undefined) {
      block.push(`> ${comment.quote}`)
    }
    block.push(escapeText(comment.text))
    parts.push(block.join('\n'))
  }
  return formatDocument(
    { revision: review.revision, verdict: review.verdict, updated: review.updated },
    parts.filter(Boolean).join('\n\n'),
  )
}

export const readReview = async (path, revision) =>
  parseReview(await readTextIfExists(path), revision)

export const writeReview = (path, review) =>
  writeFileAtomic(path, formatReview({ ...review, updated: now() }))

export const newCommentId = (comments) => {
  const taken = new Set(comments.map((comment) => comment.id))
  for (let attempt = 0; attempt < 50; attempt++) {
    const id = `c-${randomId(4)}`
    if (!taken.has(id)) return id
  }
  throw new Error('cannot find a free comment id')
}

export const cleanCommentText = (text) => {
  const clean = String(text ?? '').trim()
  if (!clean) throw new Error('a comment needs text')
  if (clean.length > 10_000) throw new Error('a comment has 10000 characters or fewer')
  return clean
}
