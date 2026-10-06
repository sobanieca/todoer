import { loadConfig } from './config.js'
import { currentBranch, git, gitText, listWorktrees } from './git.js'
import { readTextIfExists, writeFileAtomic } from './fs-utils.js'
import { join } from './paths.js'

export const DATA_DIR = '.todoer'

export const findProject = async (cwd = Deno.cwd()) => {
  const { ok, stdout } = await git(['rev-parse', '--show-toplevel'], { cwd, allowFailure: true })
  if (!ok) throw new Error('this folder is not in a git repository - run todoer in a project')
  const current = stdout.trim()
  const [main] = await listWorktrees(cwd)
  if (!main || main.bare) throw new Error('todoer needs a repository with a main worktree')
  const root = main.path
  const config = await loadConfig(root)
  const mainBranch = config.mainBranch ?? main.branch ?? 'main'
  return {
    root,
    current,
    inWorktree: current !== root,
    dataDir: join(root, DATA_DIR),
    name: root.slice(root.lastIndexOf('/') + 1),
    mainBranch,
    config,
  }
}

export const projectPath = (project, ...parts) => join(project.dataDir, ...parts)

export const excludeFromGit = async (project, pattern) => {
  const commonDir = await gitText(['rev-parse', '--path-format=absolute', '--git-common-dir'], {
    cwd: project.root,
  })
  const file = join(commonDir, 'info', 'exclude')
  const text = (await readTextIfExists(file)) ?? ''
  const lines = text.split('\n').map((line) => line.trim())
  if (lines.includes(pattern)) return false
  const prefix = text === '' || text.endsWith('\n') ? text : `${text}\n`
  await writeFileAtomic(file, `${prefix}${pattern}\n`)
  return true
}

export const branchOf = (cwd) => currentBranch(cwd)

const DATA_GITIGNORE = 'worktrees/\nruns/\n'

export const ensureDataDir = async (project) => {
  const file = join(project.dataDir, '.gitignore')
  if ((await readTextIfExists(file)) === null) await writeFileAtomic(file, DATA_GITIGNORE)
}

export const relativeToRoot = (project, path) =>
  path === project.root
    ? '.'
    : path.startsWith(`${project.root}/`)
    ? path.slice(project.root.length + 1)
    : path

export const absoluteFromRoot = (project, path) =>
  path.startsWith('/') ? path : join(project.root, path)
