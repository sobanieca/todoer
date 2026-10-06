import { requireOnlyFlags } from '../args.js'
import { CONFIG_FILE, DEFAULT_MAX_IMAGE_KB } from '../config.js'
import { exists, writeFileAtomic } from '../fs-utils.js'
import { logger } from '../logger.js'
import { ensureDataDir, findProject } from '../project.js'
import { join } from '../paths.js'

export const init = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'init')
  if (positional.length > 1) throw new Error('use: todoer init')
  const project = await findProject()
  await ensureDataDir(project)
  await Deno.mkdir(join(project.dataDir, 'todos'), { recursive: true })
  const config = join(project.root, CONFIG_FILE)
  if (await exists(config)) logger.plain(`${CONFIG_FILE} exists - kept it`)
  else {
    await writeFileAtomic(
      config,
      `${JSON.stringify({ maxImageKb: DEFAULT_MAX_IMAGE_KB }, null, 2)}\n`,
    )
    logger.plain(`Created ${config}`)
  }
  logger.plain(`Data folder: ${project.dataDir}`)
  logger.plain('Next: todoer add "<title>", todoer ui, todoer skill --init')
}
