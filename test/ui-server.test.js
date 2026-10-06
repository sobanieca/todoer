import { startUi } from '../src/ui-server.js'
import { join } from '../src/paths.js'
import { assertEquals, temporaryDir, waitFor } from './assert.js'
import { createRepo, pngOfSize, removeRepo } from './fixtures.js'

const rawStatus = async (port, host) => {
  const connection = await Deno.connect({ hostname: '127.0.0.1', port })
  const request = `GET /api/project HTTP/1.1\r\nhost: ${host}\r\nconnection: close\r\n\r\n`
  await connection.write(new TextEncoder().encode(request))
  const buffer = new Uint8Array(64)
  const size = await connection.read(buffer)
  connection.close()
  return new TextDecoder().decode(buffer.subarray(0, size)).split(' ')[1]
}

const withServer = async (config, test) => {
  const { root, project } = await createRepo(config)
  const uiDir = await temporaryDir()
  await Deno.writeTextFile(join(uiDir, 'index.html'), '<!DOCTYPE html><title>todoer</title>')
  const server = await startUi(project, { port: 0, uiDir })
  const origin = `http://localhost:${server.port}`
  const call = (path, { method = 'GET', body, headers = {} } = {}) =>
    fetch(`${origin}${path}`, {
      method,
      headers: {
        origin,
        ...(body && !(body instanceof Uint8Array) ? { 'content-type': 'application/json' } : {}),
        ...headers,
      },
      body: body instanceof Uint8Array ? body : body ? JSON.stringify(body) : undefined,
    })
  try {
    await test({ root, origin, call, port: server.port })
  } finally {
    await server.shutdown()
    await removeRepo(root)
    await Deno.remove(uiDir, { recursive: true })
  }
}

Deno.test('serves the page and the API on localhost only', async () => {
  await withServer({ maxImageKb: 1 }, async ({ origin, call, port }) => {
    const page = await call('/todos/abcd')
    assertEquals(page.status, 200)
    assertEquals(page.headers.get('x-frame-options'), 'DENY')
    assertEquals((await page.text()).includes('todoer'), true)
    const project = await (await call('/api/project')).json()
    assertEquals([project.maxImageKb, project.maxImageBytes, project.mainBranch], [1, 1000, 'main'])
    assertEquals(await rawStatus(port, `evil.example:${port}`), '421')
    assertEquals(await rawStatus(port, `localhost:${port}`), '200')
    const crossOrigin = await fetch(`${origin}/api/todos`, {
      method: 'POST',
      headers: { origin: 'http://evil.example', 'content-type': 'application/json' },
      body: '{"title":"x"}',
    })
    await crossOrigin.body?.cancel()
    assertEquals(crossOrigin.status, 403)
  })
})

Deno.test('creates a todo, attaches images and checks the limit', async () => {
  await withServer({ maxImageKb: 1 }, async ({ call }) => {
    const created = await call('/api/todos', { method: 'POST', body: { title: 'From the page' } })
    assertEquals(created.status, 201)
    const todo = await created.json()
    const small = await call(`/api/todos/${todo.id}/images?name=shot.png`, {
      method: 'POST',
      body: pngOfSize(1000),
      headers: { 'content-type': 'image/png' },
    })
    assertEquals(small.status, 201)
    const image = await small.json()
    assertEquals(image.name, '1-shot.png')
    const big = await call(`/api/todos/${todo.id}/images?name=big.png`, {
      method: 'POST',
      body: pngOfSize(1001),
      headers: { 'content-type': 'image/png' },
    })
    assertEquals(big.status, 413)
    assertEquals((await big.json()).error.includes('larger than 1 kB'), true)
    const text = await call(`/api/todos/${todo.id}/images?name=a.png`, {
      method: 'POST',
      body: new TextEncoder().encode('not an image'),
    })
    assertEquals(text.status, 415)
    await text.body?.cancel()
    const file = await call(`/files/assets/${todo.id}/1-shot.png`)
    assertEquals(file.headers.get('content-type'), 'image/png')
    assertEquals((await file.arrayBuffer()).byteLength, 1000)
    const read = await (await call(`/api/todos/${todo.id}`)).json()
    assertEquals(read.images.map((entry) => entry.name), ['1-shot.png'])
    const missing = await call('/api/todos/zzzz')
    assertEquals([missing.status, (await missing.json()).error], [
      404,
      'there is no todo zzzz - run todoer ls',
    ])
  })
})

Deno.test('builds agent prompts and refuses to run an agent without agentCommand', async () => {
  await withServer(null, async ({ call }) => {
    const a = await (await call('/api/todos', { method: 'POST', body: { title: 'A' } })).json()
    const b = await (await call('/api/todos', { method: 'POST', body: { title: 'B' } })).json()
    const together = await (await call('/api/agent/prompts', {
      method: 'POST',
      body: { todos: [a.id, b.id], mode: 'together' },
    })).json()
    assertEquals(together.prompts.length, 1)
    assertEquals(together.prompts[0].includes(`${a.id}, ${b.id} in one worktree`), true)
    const each = await (await call('/api/agent/prompts', {
      method: 'POST',
      body: { todos: [a.id, b.id], mode: 'each' },
    })).json()
    assertEquals(each.prompts.length, 2)
    const run = await call('/api/agent/run', { method: 'POST', body: { todos: [a.id] } })
    assertEquals(run.status, 400)
    assertEquals((await run.json()).error.includes('agentCommand'), true)
  })
})

Deno.test('tells the page when a file in .todoer changes', async () => {
  await withServer(null, async ({ call }) => {
    const response = await call('/api/events')
    const reader = response.body.getReader()
    let received = ''
    const reading = (async () => {
      while (!received.includes('event: change')) {
        const { value, done } = await reader.read()
        if (done) break
        received += new TextDecoder().decode(value)
      }
    })()
    await call('/api/todos', { method: 'POST', body: { title: 'Watched' } }).then((r) =>
      r.body?.cancel()
    )
    await waitFor(() => received.includes('event: change'), 5000)
    await reader.cancel()
    await reading
  })
})
