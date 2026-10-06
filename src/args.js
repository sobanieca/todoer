const FLAGS = {
  port: 'string',
  status: 'string',
  body: 'string',
  title: 'string',
  message: 'string',
  revision: 'string',
  group: 'list',
  force: 'boolean',
  ready: 'boolean',
  here: 'boolean',
  keep: 'boolean',
  all: 'boolean',
  open: 'boolean',
  json: 'boolean',
  diff: 'boolean',
  init: 'boolean',
  help: 'boolean',
  version: 'boolean',
  debug: 'boolean',
}

const ALIASES = {
  p: 'port',
  s: 'status',
  m: 'message',
  t: 'title',
  g: 'group',
  h: 'help',
  v: 'version',
}

const camelCase = (name) => name.replace(/-([a-z0-9])/g, (_, letter) => letter.toUpperCase())

export const parseArgs = (argv) => {
  const positional = []
  const flags = {}
  for (let index = 0; index < argv.length; index++) {
    const argument = argv[index]
    if (argument === '--') {
      positional.push(...argv.slice(index + 1))
      break
    }
    let name = null
    let inline = null
    if (argument.startsWith('--')) {
      const equals = argument.indexOf('=')
      name = equals === -1 ? argument.slice(2) : argument.slice(2, equals)
      inline = equals === -1 ? null : argument.slice(equals + 1)
    } else if (/^-[a-zA-Z]$/.test(argument)) {
      name = ALIASES[argument[1]]
      if (!name) throw new Error(`unknown option ${argument} - run todoer help`)
    } else {
      positional.push(argument)
      continue
    }
    const kind = FLAGS[name]
    if (!kind) throw new Error(`unknown option --${name} - run todoer help`)
    if (kind === 'boolean') {
      if (inline !== null) throw new Error(`--${name} takes no value`)
      flags[camelCase(name)] = true
      continue
    }
    const value = inline ?? argv[++index]
    if (value === undefined || (inline === null && value.startsWith('-') && value.length > 1)) {
      throw new Error(`--${name} needs a value`)
    }
    if (kind === 'list') flags[camelCase(name)] = [...(flags[camelCase(name)] ?? []), value]
    else flags[camelCase(name)] = value
  }
  return { positional, flags }
}

export const flagName = (camel) =>
  `--${Object.keys(FLAGS).find((name) => camelCase(name) === camel) ?? camel}`

export const requireOnlyFlags = (flags, allowed, command) => {
  const extra = Object.keys(flags).filter((flag) => !allowed.includes(flag) && flag !== 'debug')
  if (extra.length) {
    throw new Error(`${command} does not take ${extra.map(flagName).join(', ')}`)
  }
}

export const parsePort = (value) => {
  const port = Number(value)
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new Error(`"${value}" is not a port - use a number from 1 to 65535`)
  }
  return port
}
