import { parseArgs } from './src/args.js'
import { attach } from './src/commands/attach.js'
import { dep } from './src/commands/dep.js'
import { group } from './src/commands/group.js'
import { help } from './src/commands/help.js'
import { init } from './src/commands/init.js'
import { pr } from './src/commands/pr.js'
import { skill } from './src/commands/skill.js'
import { start } from './src/commands/start.js'
import { add, ls, rm, show, status } from './src/commands/todos.js'
import { ui } from './src/commands/ui.js'
import { version } from './src/commands/version.js'
import { logger } from './src/logger.js'

const COMMANDS = {
  add,
  attach,
  dep,
  group,
  help,
  init,
  ls,
  pr,
  rm,
  show,
  skill,
  start,
  status,
  ui,
  version,
}

try {
  const parsed = parseArgs(Deno.args)
  logger.setDebug(Boolean(parsed.flags.debug))
  const [name] = parsed.positional
  if (parsed.flags.version) version()
  else if (parsed.flags.help || name === undefined) help()
  else if (Object.hasOwn(COMMANDS, name)) await COMMANDS[name](parsed)
  else throw new Error(`unknown command "${name}" - run todoer help`)
} catch (error) {
  logger.error(error instanceof Error ? error.message : String(error))
  logger.debug(error)
  Deno.exit(1)
}
