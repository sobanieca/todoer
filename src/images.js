import { maxImageBytes } from './config.js'
import { listDirectory, removeIfExists, writeFileAtomic } from './fs-utils.js'
import { slugify } from './ids.js'
import { projectPath } from './project.js'
import { getTodo } from './todos.js'
import { join } from './paths.js'

export const IMAGE_TYPES = {
  png: 'image/png',
  jpg: 'image/jpeg',
  gif: 'image/gif',
  webp: 'image/webp',
}

const startsWith = (bytes, signature, offset = 0) =>
  signature.every((byte, index) => bytes[offset + index] === byte)

export const imageExtension = (bytes) => {
  if (startsWith(bytes, [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])) return 'png'
  if (startsWith(bytes, [0xff, 0xd8, 0xff])) return 'jpg'
  if (startsWith(bytes, [0x47, 0x49, 0x46, 0x38])) return 'gif'
  if (
    startsWith(bytes, [0x52, 0x49, 0x46, 0x46]) && startsWith(bytes, [0x57, 0x45, 0x42, 0x50], 8)
  ) {
    return 'webp'
  }
  return null
}

export class ImageError extends Error {
  constructor(message, status) {
    super(message)
    this.status = status
  }
}

export const formatKb = (bytes) => `${Math.round(bytes / 100) / 10} kB`

export const checkImage = (bytes, config) => {
  const limit = maxImageBytes(config)
  if (bytes.length > limit) {
    throw new ImageError(
      `the image has ${formatKb(bytes.length)} - the limit is ${formatKb(limit)} ` +
        `(maxImageKb in todoer.json)`,
      413,
    )
  }
  const extension = imageExtension(bytes)
  if (!extension) throw new ImageError('the file is not a PNG, JPEG, GIF or WebP image', 415)
  return extension
}

const assetsDir = (project, id) => projectPath(project, 'assets', id)

export const imageMarkdown = (todoId, name) => `![${name}](../assets/${todoId}/${name})`

export const listImages = async (project, id) => {
  const images = []
  for (const entry of await listDirectory(assetsDir(project, id))) {
    if (!entry.isFile) continue
    const extension = entry.name.slice(entry.name.lastIndexOf('.') + 1)
    if (!IMAGE_TYPES[extension]) continue
    const path = join(assetsDir(project, id), entry.name)
    const { size } = await Deno.stat(path)
    images.push({ name: entry.name, size, path, markdown: imageMarkdown(id, entry.name) })
  }
  return images
}

export const addImage = async (project, id, { name = 'image', bytes }) => {
  const todo = await getTodo(project, id)
  const extension = checkImage(bytes, project.config)
  const existing = await listImages(project, todo.id)
  const numbers = existing.map((image) => parseInt(image.name, 10)).filter(Number.isFinite)
  const next = Math.max(0, ...numbers) + 1
  const base = slugify(String(name).replace(/\.[a-z0-9]+$/i, ''), 30)
  const file = `${next}-${base}.${extension}`
  const path = join(assetsDir(project, todo.id), file)
  await writeFileAtomic(path, bytes)
  return { name: file, size: bytes.length, path, markdown: imageMarkdown(todo.id, file) }
}

export const removeImage = async (project, id, name) => {
  if (!/^[a-z0-9-]+\.(png|jpg|gif|webp)$/.test(name)) throw new ImageError('no such image', 404)
  const removed = await removeIfExists(join(assetsDir(project, id), name))
  if (!removed) throw new ImageError('no such image', 404)
}

export const imagePath = (project, id, name) => {
  if (!/^[a-z0-9]{4,8}$/.test(id) || !/^[a-z0-9-]+\.(png|jpg|gif|webp)$/.test(name)) return null
  return join(assetsDir(project, id), name)
}
