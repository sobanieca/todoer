let debugEnabled = false

let quiet = false

const format = (value) =>
  typeof value === 'string' ? value : Deno.inspect(value, { depth: 4, colors: false })

export const logger = {
  plain: (text) => {
    if (!quiet) console.log(text)
  },
  warn: (value) => {
    if (!quiet) console.error(`warning: ${format(value)}`)
  },
  error: (value) => {
    if (!quiet) console.error(`error: ${format(value)}`)
  },
  debug: (value) => {
    if (debugEnabled && !quiet) console.error(format(value))
  },
  setDebug: (value) => {
    debugEnabled = value
  },
  setQuiet: (value) => {
    quiet = value
  },
}
