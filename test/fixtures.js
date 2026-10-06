import { git } from '../src/git.js'
import { findProject } from '../src/project.js'
import { join } from '../src/paths.js'
import { temporaryDir } from './assert.js'

export const PNG_HEADER = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]

export const pngOfSize = (size) => {
  const bytes = new Uint8Array(size)
  bytes.set(PNG_HEADER)
  return bytes
}

export const writeFile = async (root, path, text) => {
  const file = join(root, path)
  await Deno.mkdir(file.slice(0, file.lastIndexOf('/')), { recursive: true })
  await Deno.writeTextFile(file, text)
}

export const run = (cwd, ...args) => git(args, { cwd })

export const createRepo = async (config = { worktreesDir: '.todoer/worktrees' }) => {
  const root = await Deno.realPath(await temporaryDir())
  await run(root, 'init', '-q', '-b', 'main')
  await run(root, 'config', 'user.email', 'test@example.com')
  await run(root, 'config', 'user.name', 'Test')
  await run(root, 'config', 'commit.gpgsign', 'false')
  await writeFile(root, 'src/app.js', 'one\ntwo\nthree\n')
  await writeFile(root, 'src/legacy/old.js', 'old\n')
  if (config) await writeFile(root, 'todoer.json', JSON.stringify(config))
  await run(root, 'add', '-A')
  await run(root, 'commit', '-qm', 'Start')
  return { root, project: await findProject(root) }
}

export const removeRepo = (root) => Deno.remove(root, { recursive: true })
