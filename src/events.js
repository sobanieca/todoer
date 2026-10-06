import { logger } from './logger.js'

const encoder = new TextEncoder()

export const createEvents = (dataDir) => {
  const clients = new Set()
  let watcher = null
  let timer = null
  const broadcast = () => {
    for (const controller of clients) {
      try {
        controller.enqueue(encoder.encode('event: change\ndata: {}\n\n'))
      } catch {
        clients.delete(controller)
      }
    }
  }
  const start = async () => {
    await Deno.mkdir(dataDir, { recursive: true })
    watcher = Deno.watchFs(dataDir, { recursive: true })
    ;(async () => {
      try {
        for await (const event of watcher) {
          if (event.paths.every((path) => path.endsWith('.tmp') || path.includes('/worktrees/'))) {
            continue
          }
          clearTimeout(timer)
          timer = setTimeout(broadcast, 150)
        }
      } catch (error) {
        logger.debug(error)
      }
    })()
  }
  const subscribe = () => {
    let current
    let ping
    const body = new ReadableStream({
      start: (controller) => {
        current = controller
        clients.add(controller)
        controller.enqueue(encoder.encode('retry: 2000\n\n'))
        ping = setInterval(() => {
          try {
            controller.enqueue(encoder.encode(': ping\n\n'))
          } catch {
            clearInterval(ping)
          }
        }, 25_000)
      },
      cancel: () => {
        clearInterval(ping)
        clients.delete(current)
      },
    })
    return new Response(body, {
      headers: { 'content-type': 'text/event-stream', 'cache-control': 'no-store' },
    })
  }
  const close = () => {
    clearTimeout(timer)
    watcher?.close()
    for (const controller of clients) {
      try {
        controller.close()
      } catch {
        clients.delete(controller)
      }
    }
    clients.clear()
  }
  return { start, subscribe, close }
}
