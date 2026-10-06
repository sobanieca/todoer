export const assert = (condition, message = 'assertion failed') => {
  if (!condition) throw new Error(message)
}

export const assertEquals = (actual, expected, message) => {
  const left = JSON.stringify(actual)
  const right = JSON.stringify(expected)
  if (left !== right) {
    throw new Error(`${message ? `${message}: ` : ''}expected ${right}, got ${left}`)
  }
}

export const assertRejects = async (task, pattern) => {
  try {
    await task()
  } catch (error) {
    if (pattern && !pattern.test(error.message)) {
      throw new Error(`the error "${error.message}" does not match ${pattern}`)
    }
    return error
  }
  throw new Error('expected a rejection')
}

export const assertThrows = (task, pattern) => {
  try {
    task()
  } catch (error) {
    if (pattern && !pattern.test(error.message)) {
      throw new Error(`the error "${error.message}" does not match ${pattern}`)
    }
    return error
  }
  throw new Error('expected an error')
}

export const temporaryDir = () => Deno.makeTempDir({ prefix: 'todoer-test-' })

export const waitFor = async (check, timeoutMs = 10_000) => {
  const start = Date.now()
  while (true) {
    const value = await check()
    if (value) return value
    if (Date.now() - start > timeoutMs) throw new Error('timed out')
    await new Promise((resolve) => setTimeout(resolve, 50))
  }
}
