import { implementPrompt, reviewPrompt, runAgent } from './agent.js'
import { maxImageBytes } from './config.js'
import {
  addDependency,
  assertNotBlocked,
  blockersOf,
  deleteTodo,
  isReady,
  loadGraph,
  removeDependency,
} from './dependencies.js'
import { addToGroup, listGroups, removeFromGroup, removeGroup, renameGroup } from './groups.js'
import { addImage, IMAGE_TYPES, ImageError, imagePath, listImages, removeImage } from './images.js'
import { logger } from './logger.js'
import { createTodo, getTodo, listTodos, NotFoundError, STATUSES, updateTodo } from './todos.js'
import {
  addComment,
  closePr,
  getPr,
  listPrs,
  mergePr,
  readPrDetails,
  removeComment,
  setVerdict,
  updateComment,
} from './prs.js'
import { serveUi } from './static-files.js'
import { version } from './version.js'

const SECURITY_HEADERS = {
  'content-security-policy': [
    "default-src 'self'",
    "script-src 'self'",
    "style-src 'self'",
    "img-src 'self' data: blob:",
    "connect-src 'self'",
    "object-src 'none'",
    "base-uri 'none'",
    "form-action 'none'",
    "frame-ancestors 'none'",
  ].join('; '),
  'x-content-type-options': 'nosniff',
  'x-frame-options': 'DENY',
  'referrer-policy': 'no-referrer',
  'cross-origin-opener-policy': 'same-origin',
  'cross-origin-resource-policy': 'same-origin',
}

const SAFE_METHODS = ['GET', 'HEAD']

export class HttpError extends Error {
  constructor(status, message) {
    super(message)
    this.status = status
  }
}

export const allowedHosts = (port) =>
  new Set([`localhost:${port}`, `127.0.0.1:${port}`, `[::1]:${port}`])

const json = (value, status = 200) =>
  new Response(JSON.stringify(value), {
    status,
    headers: { 'content-type': 'application/json', 'cache-control': 'no-store' },
  })

const readJsonBody = async (request) => {
  try {
    const value = await request.json()
    if (value === null || typeof value !== 'object' || Array.isArray(value)) throw new Error()
    return value
  } catch {
    throw new HttpError(400, 'the body is not a JSON object')
  }
}

const readLimitedBody = async (request, limit) => {
  const declared = Number(request.headers.get('content-length'))
  if (Number.isFinite(declared) && declared > limit) return null
  const chunks = []
  let size = 0
  for await (const chunk of request.body ?? []) {
    size += chunk.length
    if (size > limit) return null
    chunks.push(chunk)
  }
  const bytes = new Uint8Array(size)
  let offset = 0
  for (const chunk of chunks) {
    bytes.set(chunk, offset)
    offset += chunk.length
  }
  return bytes
}

const withHeaders = (response) => {
  const headers = new Headers(response.headers)
  for (const [name, value] of Object.entries(SECURITY_HEADERS)) {
    if (!headers.has(name)) headers.set(name, value)
  }
  return new Response(response.body, { status: response.status, headers })
}

const statusOf = (error) => {
  if (error instanceof HttpError || error instanceof ImageError) return error.status
  if (error instanceof NotFoundError) return 404
  return 400
}

const todoView = async (project, todo) => ({
  ...todo,
  images: (await listImages(project, todo.id)).map(({ path: _path, ...image }) => image),
})

const route = (method, pattern, handler) => {
  const regex = new RegExp(`^${pattern.replace(/:([a-zA-Z]+)/g, '(?<$1>[^/]+)')}$`)
  return { method, regex, handler }
}

const prompts = (ids, mode) => {
  if (!Array.isArray(ids) || !ids.length || !ids.every((id) => typeof id === 'string')) {
    throw new HttpError(400, 'name one or more todos in "todos"')
  }
  return mode === 'each' ? ids.map((id) => implementPrompt([id])) : [implementPrompt(ids)]
}

const createRoutes = (project) => [
  route('GET', '/api/project', () =>
    json({
      name: project.name,
      root: project.root,
      mainBranch: project.mainBranch,
      maxImageKb: project.config.maxImageKb,
      maxImageBytes: maxImageBytes(project.config),
      agentCommand: project.config.agentCommand,
      statuses: STATUSES,
      version,
    })),
  route('GET', '/api/todos', async () => json({ todos: await listTodos(project) })),
  route('POST', '/api/todos', async ({ request }) => {
    const body = await readJsonBody(request)
    const todo = await createTodo(project, {
      title: body.title,
      body: body.body ?? '',
      groups: body.groups ?? [],
    })
    return json(await todoView(project, todo), 201)
  }),
  route(
    'GET',
    '/api/todos/:id',
    async ({ params }) => json(await todoView(project, await getTodo(project, params.id))),
  ),
  route('PATCH', '/api/todos/:id', async ({ params, request }) => {
    const body = await readJsonBody(request)
    const changes = {}
    for (const key of ['title', 'body', 'status', 'groups']) {
      if (body[key] !== undefined) changes[key] = body[key]
    }
    return json(await todoView(project, await updateTodo(project, params.id, changes)))
  }),
  route('DELETE', '/api/todos/:id', async ({ params }) => {
    await deleteTodo(project, params.id)
    return json({ ok: true })
  }),
  route('POST', '/api/todos/:id/images', async ({ params, request, url }) => {
    const limit = maxImageBytes(project.config)
    const bytes = await readLimitedBody(request, limit)
    if (bytes === null) {
      throw new HttpError(
        413,
        `the image is larger than ${project.config.maxImageKb} kB (maxImageKb in todoer.json)`,
      )
    }
    const image = await addImage(project, params.id, {
      name: url.searchParams.get('name') ?? 'image',
      bytes,
    })
    const { path: _path, ...view } = image
    return json(view, 201)
  }),
  route('DELETE', '/api/todos/:id/images/:name', async ({ params }) => {
    await removeImage(project, params.id, params.name)
    return json({ ok: true })
  }),
  route('POST', '/api/agent/prompts', async ({ request }) => {
    const body = await readJsonBody(request)
    if (body.pr) {
      await getPr(project, body.pr)
      return json({ prompts: [reviewPrompt(body.pr)] })
    }
    for (const id of body.todos ?? []) await getTodo(project, id)
    return json({ prompts: prompts(body.todos, body.mode) })
  }),
  route('POST', '/api/agent/run', async ({ request }) => {
    const body = await readJsonBody(request)
    let list
    if (body.pr) {
      await getPr(project, body.pr)
      list = [reviewPrompt(body.pr)]
    } else {
      for (const id of body.todos ?? []) await getTodo(project, id)
      list = prompts(body.todos, body.mode)
    }
    const runs = []
    for (const prompt of list) runs.push(await runAgent(project, prompt))
    return json({ runs })
  }),
  route('GET', '/api/prs', async () => json({ prs: await listPrs(project) })),
  route('GET', '/api/graph', async () => {
    const nodes = await loadGraph(project)
    return json({
      nodes: [...nodes.values()].map((node) => ({
        ...node,
        blockedBy: blockersOf(nodes, node.key),
        ready: isReady(nodes, node.key),
      })),
    })
  }),
  route('POST', '/api/dependencies', async ({ request }) => {
    const body = await readJsonBody(request)
    return json(await addDependency(project, body.from, body.to), 201)
  }),
  route('POST', '/api/dependencies/remove', async ({ request }) => {
    const body = await readJsonBody(request)
    return json(await removeDependency(project, body.from, body.to))
  }),
  route('GET', '/api/groups', async () => json({ groups: await listGroups(project) })),
  route('POST', '/api/groups/add', async ({ request }) => {
    const body = await readJsonBody(request)
    return json({ count: await addToGroup(project, body.name, body.todos ?? []) })
  }),
  route('POST', '/api/groups/remove-todos', async ({ request }) => {
    const body = await readJsonBody(request)
    return json({ count: await removeFromGroup(project, body.name, body.todos ?? []) })
  }),
  route('POST', '/api/groups/rename', async ({ request }) => {
    const body = await readJsonBody(request)
    return json({ count: await renameGroup(project, body.from, body.to) })
  }),
  route('POST', '/api/groups/remove', async ({ request }) => {
    const body = await readJsonBody(request)
    return json({ count: await removeGroup(project, body.name) })
  }),
  route('GET', '/api/prs/:id', async ({ params, url }) => {
    const revision = url.searchParams.get('revision')
    const { diff: _diff, ...details } = await readPrDetails(
      project,
      params.id,
      revision ?? undefined,
    )
    return json(details)
  }),
  route('POST', '/api/prs/:id/comments', async ({ params, request }) => {
    const body = await readJsonBody(request)
    return json(await addComment(project, params.id, body), 201)
  }),
  route('PATCH', '/api/prs/:id/comments/:comment', async ({ params, request }) => {
    const body = await readJsonBody(request)
    return json(await updateComment(project, params.id, params.comment, body))
  }),
  route('DELETE', '/api/prs/:id/comments/:comment', async ({ params }) => {
    await removeComment(project, params.id, params.comment)
    return json({ ok: true })
  }),
  route('POST', '/api/prs/:id/review', async ({ params, request }) => {
    const body = await readJsonBody(request)
    const review = await setVerdict(project, params.id, body)
    const prompt = review.verdict === 'changes-requested' ? reviewPrompt(params.id) : null
    return json({ review, prompt })
  }),
  route('POST', '/api/prs/:id/merge', async ({ params, request }) => {
    const body = await readJsonBody(request)
    if (body.force !== true) {
      await assertNotBlocked(project, [`pr:${params.id}`], `merge pull request ${params.id}`)
    }
    return json(await mergePr(project, params.id, { keep: body.keep === true }))
  }),
  route(
    'POST',
    '/api/prs/:id/close',
    async ({ params }) => json(await closePr(project, params.id)),
  ),
]

const serveImage = async (project, id, name) => {
  const path = imagePath(project, id, name)
  if (!path) return null
  try {
    const bytes = await Deno.readFile(path)
    return new Response(bytes, {
      headers: {
        'content-type': IMAGE_TYPES[name.slice(name.lastIndexOf('.') + 1)],
        'cache-control': 'no-cache',
        'content-security-policy': "default-src 'none'",
      },
    })
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return null
    throw error
  }
}

export const createHandler = ({ project, uiDir, hosts, events }) => {
  const routes = createRoutes(project)
  const handle = async (request) => {
    const url = new URL(request.url)
    if (!hosts().has(request.headers.get('host') ?? '')) {
      return new Response('unknown host', { status: 421 })
    }
    const path = url.pathname
    if (path.startsWith('/api/')) {
      if (request.headers.get('sec-fetch-site') === 'cross-site') {
        return json({ error: 'cross-site requests are not allowed' }, 403)
      }
      if (!SAFE_METHODS.includes(request.method)) {
        const origin = request.headers.get('origin')
        if (!origin || !hosts().has(origin.replace(/^https?:\/\//, ''))) {
          return json({ error: 'the origin of the request is not this page' }, 403)
        }
      }
      if (path === '/api/events' && request.method === 'GET') return events.subscribe()
      for (const entry of routes) {
        const match = path.match(entry.regex)
        if (!match || entry.method !== request.method) continue
        const params = Object.fromEntries(
          Object.entries(match.groups ?? {}).map((
            [key, value],
          ) => [key, decodeURIComponent(value)]),
        )
        try {
          return await entry.handler({ request, url, params })
        } catch (error) {
          if (error instanceof Error && !(error instanceof TypeError)) {
            return json({ error: error.message }, statusOf(error))
          }
          logger.error(error)
          return json({ error: 'internal error' }, 500)
        }
      }
      return json({ error: 'not found' }, 404)
    }
    const asset = path.match(/^\/files\/assets\/([^/]+)\/([^/]+)$/)
    if (asset && SAFE_METHODS.includes(request.method)) {
      return (await serveImage(project, asset[1], decodeURIComponent(asset[2]))) ??
        new Response('not found', { status: 404 })
    }
    if (!SAFE_METHODS.includes(request.method)) {
      return new Response('method not allowed', { status: 405 })
    }
    return serveUi(uiDir, path)
  }
  return async (request) => withHeaders(await handle(request))
}
