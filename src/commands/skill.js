import { requireOnlyFlags } from '../args.js'
import { exists } from '../fs-utils.js'
import { logger } from '../logger.js'
import { dirname, join } from '../paths.js'
import { findProject } from '../project.js'
import { SKILL_NAME, SKILL_TARGETS, skillContent, workflowText } from '../skill.js'

export const installSkill = async (root) => {
  const files = SKILL_TARGETS.map((target) => ({
    ...target,
    path: join(target.dir, SKILL_NAME, 'SKILL.md'),
  }))
  for (const { path } of files) {
    const file = join(root, path)
    if (await exists(file)) throw new Error(`${file} exists - remove it to write the skill again`)
  }
  for (const { path } of files) {
    const file = join(root, path)
    await Deno.mkdir(dirname(file), { recursive: true })
    await Deno.writeTextFile(file, skillContent)
  }
  return files
}

export const skill = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['init'], 'skill')
  if (positional.length > 1) throw new Error('use: todoer skill [--init]')
  if (!flags.init) return logger.plain(workflowText)
  const project = await findProject()
  const files = await installSkill(project.root)
  logger.plain(`Project root: ${project.root}`)
  for (const { path, readBy } of files) logger.plain(`Created ${path}  (${readBy})`)
  logger.plain('')
  logger.plain('Try: /todoer implement 7f3a')
}
