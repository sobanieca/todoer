import { join } from './paths.js'
import { readTextIfExists } from './fs-utils.js'

export const CONFIG_FILE = 'todoer.json'

export const DEFAULT_MAX_IMAGE_KB = 150

export const DEFAULT_PORT = 4507

const KEYS = {
  maxImageKb: (value) => Number.isFinite(value) && value > 0,
  mainBranch: (value) => typeof value === 'string' && value.length > 0,
  worktreesDir: (value) => typeof value === 'string' && value.length > 0 && !value.startsWith('/'),
  agentCommand: (value) =>
    Array.isArray(value) && value.length > 0 && value.every((part) => typeof part === 'string'),
  port: (value) => Number.isInteger(value) && value > 0 && value < 65536,
}

const RULES = {
  maxImageKb: 'a number of kilobytes above 0',
  mainBranch: 'a branch name',
  worktreesDir: 'a folder relative to the project root',
  agentCommand: 'an array of strings, like ["claude", "{prompt}"]',
  port: 'a port from 1 to 65535',
}

export const parseConfig = (text) => {
  let value
  try {
    value = JSON.parse(text)
  } catch (error) {
    throw new Error(`${CONFIG_FILE} is not valid JSON: ${error.message}`)
  }
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`${CONFIG_FILE} must hold an object`)
  }
  for (const [key, entry] of Object.entries(value)) {
    if (!KEYS[key]) throw new Error(`${CONFIG_FILE} has an unknown key "${key}"`)
    if (!KEYS[key](entry)) {
      throw new Error(`invalid ${CONFIG_FILE} - "${key}" must be ${RULES[key]}`)
    }
  }
  return value
}

export const defaultWorktreesDir = (env = Deno.env.toObject()) =>
  env.CLAUDECODE ? '.claude/worktrees' : '.todoer/worktrees'

export const loadConfig = async (root) => {
  const text = await readTextIfExists(join(root, CONFIG_FILE))
  const config = text === null ? {} : parseConfig(text)
  return {
    maxImageKb: config.maxImageKb ?? DEFAULT_MAX_IMAGE_KB,
    mainBranch: config.mainBranch ?? null,
    worktreesDir: config.worktreesDir ?? defaultWorktreesDir(),
    agentCommand: config.agentCommand ?? null,
    port: config.port ?? DEFAULT_PORT,
  }
}

export const maxImageBytes = (config) => Math.round(config.maxImageKb * 1000)
