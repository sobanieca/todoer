import { requireOnlyFlags } from '../args.js'
import { blockersOf, loadGraph } from '../dependencies.js'
import { addToGroup, listGroups, removeFromGroup, removeGroup, renameGroup } from '../groups.js'
import { logger } from '../logger.js'
import { findProject } from '../project.js'
import { cleanGroupName, NotFoundError } from '../todos.js'

const printJson = (value) => console.log(JSON.stringify(value, null, 2))

const list = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['json'], 'group ls')
  if (positional.length > 2) throw new Error('use: todoer group ls [--json]')
  const groups = await listGroups(await findProject())
  if (flags.json) return printJson(groups)
  if (!groups.length) return logger.plain('No groups. Add one with: todoer group add "<name>" <id>')
  for (const group of groups) {
    const detail = [
      `${group.done}/${group.total} done`,
      group.ready ? `${group.ready} ready` : null,
      group.blocked ? `${group.blocked} blocked` : null,
      group.openPrs ? `${group.openPrs} open pr` : null,
    ].filter(Boolean).join(', ')
    logger.plain(`${String(group.percent).padStart(3)}%  ${group.name}  (${detail})`)
  }
}

const show = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['json'], 'group show')
  const name = positional.slice(2).join(' ')
  if (!name) throw new Error('use: todoer group show "<name>"')
  const clean = cleanGroupName(name)
  const project = await findProject()
  const nodes = await loadGraph(project)
  const group = (await listGroups(project)).find((entry) => entry.name === clean)
  if (!group) throw new NotFoundError(`there is no group "${clean}" - run todoer group ls`)
  const todos = [...nodes.values()]
    .filter((node) => node.kind === 'todo' && node.groups.includes(clean))
    .map((node) => ({ ...node, blockedBy: blockersOf(nodes, node.key) }))
  if (flags.json) return printJson({ ...group, todos })
  logger.plain(`${group.name} - ${group.done} of ${group.total} done (${group.percent}%)`)
  for (const todo of todos) {
    const waits = todo.blockedBy.length
      ? `  waits for ${todo.blockedBy.map((key) => key.split(':')[1]).join(', ')}`
      : ''
    const pr = todo.pr ? `  ${todo.pr.replace(':', ' ')}` : ''
    logger.plain(`  ${todo.id}  ${todo.status.padEnd(11)}  ${todo.title}${pr}${waits}`)
  }
}

const add = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'group add')
  const [, , name, ...ids] = positional
  if (!name || !ids.length) throw new Error('use: todoer group add "<name>" <id> [<id> ...]')
  const count = await addToGroup(await findProject(), name, ids)
  logger.plain(`Added ${count} todo(s) to the group "${cleanGroupName(name)}"`)
}

const remove = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'group rm')
  const [, , name, ...ids] = positional
  if (!name) throw new Error('use: todoer group rm "<name>" [<id> ...]')
  const project = await findProject()
  if (ids.length) {
    await removeFromGroup(project, name, ids)
    return logger.plain(`Removed ${ids.length} todo(s) from the group "${cleanGroupName(name)}"`)
  }
  const count = await removeGroup(project, name)
  logger.plain(`Removed the group "${cleanGroupName(name)}" from ${count} todo(s)`)
}

const rename = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'group rename')
  const [, , from, to] = positional
  if (!from || !to || positional.length > 4) {
    throw new Error('use: todoer group rename "<old name>" "<new name>"')
  }
  const count = await renameGroup(await findProject(), from, to)
  logger.plain(`Renamed the group in ${count} todo(s)`)
}

const SUBCOMMANDS = { ls: list, show, add, rm: remove, rename }

export const group = async (parsed) => {
  const [, name] = parsed.positional
  if (!name) throw new Error(`use: todoer group <${Object.keys(SUBCOMMANDS).join('|')}>`)
  if (!Object.hasOwn(SUBCOMMANDS, name)) {
    throw new Error(`unknown command "group ${name}" - run todoer help`)
  }
  await SUBCOMMANDS[name](parsed)
}
