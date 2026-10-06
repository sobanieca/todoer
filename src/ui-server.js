import { uiIsBuilt } from './static-files.js'
import { createEvents } from './events.js'

export const defaultUiDir = () =>
  decodeURIComponent(new URL('../ui/dist', import.meta.url).pathname)

const listen = (options, handler) => Deno.serve({ ...options, onListen: () => {} }, handler)

export const startUi = async (project, { port, uiDir = defaultUiDir(), fallback = true } = {}) => {
  if (!(await uiIsBuilt(uiDir))) {
    throw new Error(`the page is missing in ${uiDir} - run: deno task ui:build`)
  }
  const { allowedHosts, createHandler } = await import('./ui-api.js')
  const events = createEvents(project.dataDir)
  await events.start()
  let hosts = allowedHosts(port)
  const handler = createHandler({ project, uiDir, hosts: () => hosts, events })
  let main
  try {
    main = listen({ hostname: '127.0.0.1', port }, handler)
  } catch (error) {
    if (!(error instanceof Deno.errors.AddrInUse)) throw error
    if (!fallback) throw new Error(`the port ${port} is in use - choose another one with --port`)
    main = listen({ hostname: '127.0.0.1', port: 0 }, handler)
  }
  const bound = main.addr.port
  hosts = allowedHosts(bound)
  const servers = [main]
  try {
    servers.push(listen({ hostname: '::1', port: bound }, handler))
  } catch {
    servers.length = 1
  }
  return {
    port: bound,
    url: `http://localhost:${bound}`,
    shutdown: async () => {
      events.close()
      await Promise.all(servers.map((server) => server.shutdown()))
    },
  }
}
