const unquote = (path) => {
  if (!path.startsWith('"')) return path
  const inner = path.slice(1, -1)
  const bytes = []
  for (let index = 0; index < inner.length; index++) {
    const char = inner[index]
    if (char !== '\\') {
      bytes.push(...new TextEncoder().encode(char))
      continue
    }
    const next = inner[++index]
    if (/[0-7]/.test(next)) {
      bytes.push(parseInt(inner.slice(index, index + 3), 8))
      index += 2
    } else {
      bytes.push({ n: 10, t: 9, r: 13, '"': 34, '\\': 92 }[next] ?? next.charCodeAt(0))
    }
  }
  return new TextDecoder().decode(new Uint8Array(bytes))
}

const stripPrefix = (path) => {
  const clean = unquote(path)
  if (clean === '/dev/null') return null
  return clean.replace(/^[ab]\//, '')
}

const headerPaths = (line) => {
  const rest = line.slice('diff --git '.length)
  if (rest.startsWith('"')) {
    const match = rest.match(/^("(?:[^"\\]|\\.)*")\s+(.*)$/)
    if (match) return [stripPrefix(match[1]), stripPrefix(match[2])]
  }
  const length = (rest.length - 5) / 2
  if (Number.isInteger(length) && rest.slice(2, 2 + length) === rest.slice(length + 5)) {
    return [rest.slice(2, 2 + length), rest.slice(length + 5)]
  }
  const split = rest.indexOf(' b/')
  return split === -1 ? [null, null] : [stripPrefix(rest.slice(0, split)), rest.slice(split + 3)]
}

const HUNK = /^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@(.*)$/

export const parseDiff = (text) => {
  const files = []
  let file = null
  let hunk = null
  let oldLine = 0
  let newLine = 0
  const finish = () => {
    if (!file) return
    file.path = file.newPath ?? file.oldPath
    if (
      file.status === 'modified' && file.oldPath && file.newPath && file.oldPath !== file.newPath
    ) {
      file.status = 'renamed'
    }
    files.push(file)
  }
  for (const line of text.replace(/\r\n/g, '\n').split('\n')) {
    if (line.startsWith('diff --git ')) {
      finish()
      const [oldPath, newPath] = headerPaths(line)
      file = {
        path: null,
        oldPath,
        newPath,
        status: 'modified',
        binary: false,
        additions: 0,
        deletions: 0,
        hunks: [],
      }
      hunk = null
      continue
    }
    if (!file) continue
    if (hunk) {
      const kind = line[0]
      if (kind === '+') {
        hunk.lines.push({ kind: 'add', old: null, new: newLine++, text: line.slice(1) })
        file.additions++
        continue
      }
      if (kind === '-') {
        hunk.lines.push({ kind: 'del', old: oldLine++, new: null, text: line.slice(1) })
        file.deletions++
        continue
      }
      if (kind === ' ') {
        hunk.lines.push({ kind: 'context', old: oldLine++, new: newLine++, text: line.slice(1) })
        continue
      }
      if (kind === '\\') {
        const last = hunk.lines.at(-1)
        if (last) last.noNewline = true
        continue
      }
    }
    const match = line.match(HUNK)
    if (match) {
      oldLine = Number(match[1])
      newLine = Number(match[3])
      hunk = { header: line, context: match[5].trim(), lines: [] }
      file.hunks.push(hunk)
      continue
    }
    hunk = null
    if (line.startsWith('new file mode')) file.status = 'added'
    else if (line.startsWith('deleted file mode')) file.status = 'deleted'
    else if (line.startsWith('rename from ')) file.oldPath = unquote(line.slice(12))
    else if (line.startsWith('rename to ')) file.newPath = unquote(line.slice(10))
    else if (line.startsWith('copy from ')) file.oldPath = unquote(line.slice(10))
    else if (line.startsWith('copy to ')) file.newPath = unquote(line.slice(8))
    else if (line.startsWith('Binary files ') || line === 'GIT binary patch') file.binary = true
    else if (line.startsWith('--- ')) file.oldPath = stripPrefix(line.slice(4))
    else if (line.startsWith('+++ ')) file.newPath = stripPrefix(line.slice(4))
  }
  finish()
  for (const entry of files) {
    if (entry.status === 'added') entry.oldPath = null
    if (entry.status === 'deleted') entry.newPath = null
    entry.path = entry.newPath ?? entry.oldPath
  }
  return files
}

export const lineText = (files, target) => {
  const file = files.find((entry) => entry.path === target.path || entry.oldPath === target.path)
  if (!file) return null
  for (const hunk of file.hunks) {
    for (const line of hunk.lines) {
      if (target.side === 'old' ? line.old === target.line : line.new === target.line) {
        return line.text
      }
    }
  }
  return null
}
