import { dirname } from './paths.js'

export const writeFileAtomic = async (path, data) => {
  await Deno.mkdir(dirname(path), { recursive: true })
  const temporary = `${path}.${crypto.randomUUID().slice(0, 8)}.tmp`
  if (typeof data === 'string') await Deno.writeTextFile(temporary, data)
  else await Deno.writeFile(temporary, data)
  await Deno.rename(temporary, path)
}

export const readTextIfExists = async (path) => {
  try {
    return await Deno.readTextFile(path)
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return null
    throw error
  }
}

export const removeIfExists = async (path, options = {}) => {
  try {
    await Deno.remove(path, options)
    return true
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return false
    throw error
  }
}

export const listDirectory = async (path) => {
  const entries = []
  try {
    for await (const entry of Deno.readDir(path)) entries.push(entry)
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return []
    throw error
  }
  return entries.sort((a, b) => a.name.localeCompare(b.name))
}

export const exists = async (path) => {
  try {
    await Deno.stat(path)
    return true
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return false
    throw error
  }
}
