import { parsePort, requireOnlyFlags } from '../args.js'
import { logger } from '../logger.js'
import { findProject } from '../project.js'
import { startUi } from '../ui-server.js'

const openBrowser = async (url) => {
  const command = Deno.build.os === 'darwin'
    ? 'open'
    : Deno.build.os === 'windows'
    ? 'explorer'
    : 'xdg-open'
  try {
    await new Deno.Command(command, { args: [url], stdout: 'null', stderr: 'null' }).output()
  } catch {
    logger.warn(`cannot open a browser - go to ${url}`)
  }
}

export const ui = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['port', 'open'], 'ui')
  if (positional.length > 1) throw new Error('use: todoer ui [--port <port>] [--open]')
  const project = await findProject()
  const port = flags.port ? parsePort(flags.port) : project.config.port
  const server = await startUi(project, { port, fallback: !flags.port })
  if (server.port !== port) logger.plain(`The port ${port} is in use - took ${server.port}`)
  logger.plain(`todoer for ${project.root}`)
  logger.plain(`Open ${server.url}  (Ctrl+C stops it)`)
  if (flags.open) await openBrowser(server.url)
  const stop = async () => {
    await server.shutdown()
    Deno.exit(0)
  }
  Deno.addSignalListener('SIGINT', stop)
  if (Deno.build.os !== 'windows') Deno.addSignalListener('SIGTERM', stop)
  await new Promise(() => {})
}
