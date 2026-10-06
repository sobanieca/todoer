import { requireOnlyFlags } from '../args.js'
import {
  addDependency,
  blockersOf,
  describe,
  loadGraph,
  removeDependency,
  resolveRef,
  treeLines,
} from '../dependencies.js'
import { logger } from '../logger.js'
import { findProject } from '../project.js'

const add = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'dep add')
  const [, , item, ...others] = positional
  if (!item || !others.length) throw new Error('use: todoer dep add <id> <depends on id> [...]')
  const project = await findProject()
  for (const other of others) {
    const { from, to, added } = await addDependency(project, item, other)
    const nodes = await loadGraph(project)
    logger.plain(
      `${added ? 'Added' : 'Kept'}: ${describe(nodes, from)} depends on ${describe(nodes, to)}`,
    )
  }
}

const remove = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'dep rm')
  const [, , item, ...others] = positional
  if (!item || !others.length) throw new Error('use: todoer dep rm <id> <depends on id> [...]')
  const project = await findProject()
  for (const other of others) {
    const { from, to } = await removeDependency(project, item, other)
    logger.plain(`Removed: ${from} no longer depends on ${to}`)
  }
}

const show = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['json'], 'dep show')
  const [, , item] = positional
  if (!item || positional.length > 3) throw new Error('use: todoer dep show <id> [--json]')
  const nodes = await loadGraph(await findProject())
  const key = resolveRef(nodes, item)
  const node = nodes.get(key)
  const blockedBy = blockersOf(nodes, key)
  if (flags.json) {
    return console.log(JSON.stringify({ ...node, blockedBy }, null, 2))
  }
  const dependsOn = treeLines(nodes, key, 'dependsOn')
  const neededBy = treeLines(nodes, key, 'neededBy')
  logger.plain(
    [
      describe(nodes, key),
      blockedBy.length
        ? `Blocked - wait for: ${blockedBy.map((entry) => describe(nodes, entry)).join(', ')}`
        : 'Not blocked.',
      '',
      'Depends on:',
      ...(dependsOn.length ? dependsOn.map((line) => `  ${line}`) : ['  nothing']),
      '',
      'Needed by:',
      ...(neededBy.length ? neededBy.map((line) => `  ${line}`) : ['  nothing']),
    ].join('\n'),
  )
}

const SUBCOMMANDS = { add, rm: remove, show }

export const dep = async (parsed) => {
  const [, name] = parsed.positional
  if (!name) throw new Error(`use: todoer dep <${Object.keys(SUBCOMMANDS).join('|')}>`)
  if (!Object.hasOwn(SUBCOMMANDS, name)) {
    throw new Error(`unknown command "dep ${name}" - run todoer help`)
  }
  await SUBCOMMANDS[name](parsed)
}
