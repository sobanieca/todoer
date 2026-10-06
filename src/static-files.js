import { join } from './paths.js'

const TYPES = {
  html: 'text/html; charset=utf-8',
  js: 'text/javascript; charset=utf-8',
  css: 'text/css; charset=utf-8',
  json: 'application/json',
  webmanifest: 'application/manifest+json',
  txt: 'text/plain; charset=utf-8',
  svg: 'image/svg+xml',
  png: 'image/png',
  ico: 'image/x-icon',
  woff2: 'font/woff2',
  webp: 'image/webp',
  jpg: 'image/jpeg',
  gif: 'image/gif',
}

const IMMUTABLE = 'public, max-age=31536000, immutable'

const extensionOf = (path) => {
  const name = path.slice(path.lastIndexOf('/') + 1)
  const dot = name.lastIndexOf('.')
  return dot > 0 ? name.slice(dot + 1).toLowerCase() : ''
}

const readFile = async (path) => {
  try {
    return await Deno.readFile(path)
  } catch (error) {
    if (error instanceof Deno.errors.NotFound || error instanceof Deno.errors.IsADirectory) {
      return null
    }
    throw error
  }
}

const notFound = () =>
  new Response('not found', {
    status: 404,
    headers: { 'content-type': 'text/plain; charset=utf-8', 'cache-control': 'no-cache' },
  })

const fileResponse = (bytes, path) =>
  new Response(bytes, {
    headers: {
      'content-type': TYPES[extensionOf(path)] ?? 'application/octet-stream',
      'cache-control': path.startsWith('/chunks/') ? IMMUTABLE : 'no-cache',
    },
  })

export const uiIsBuilt = async (root) => (await readFile(join(root, 'index.html'))) !== null

export const serveUi = async (root, pathname) => {
  let path
  try {
    path = decodeURIComponent(pathname)
  } catch {
    return notFound()
  }
  const parts = path.split('/').slice(1)
  if (path.includes('\0') || parts.some((part) => part.startsWith('.'))) return notFound()
  if (path !== '/' && !path.endsWith('/')) {
    const bytes = await readFile(join(root, path))
    if (bytes) return fileResponse(bytes, path)
    if (path.startsWith('/chunks/') || extensionOf(path)) return notFound()
  }
  const index = await readFile(join(root, 'index.html'))
  return index ? fileResponse(index, '/index.html') : notFound()
}
