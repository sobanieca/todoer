const decoder = new TextDecoder()

export class GitError extends Error {
  constructor(args, stderr) {
    super(`git ${args[0]} failed: ${stderr.trim() || 'no output'}`)
    this.stderr = stderr
  }
}

export const git = async (args, { cwd, allowFailure = false } = {}) => {
  const command = new Deno.Command('git', {
    args: ['-c', 'core.quotePath=false', ...args],
    cwd,
    stdin: 'null',
    stdout: 'piped',
    stderr: 'piped',
  })
  let output
  try {
    output = await command.output()
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      throw new Error('git is not installed - todoer needs git on the PATH')
    }
    throw error
  }
  const stdout = decoder.decode(output.stdout)
  const stderr = decoder.decode(output.stderr)
  if (!output.success && !allowFailure) throw new GitError(args, stderr)
  return { ok: output.success, stdout, stderr }
}

export const gitText = async (args, options) => (await git(args, options)).stdout.trim()

export const parseWorktrees = (porcelain) => {
  const worktrees = []
  let current = null
  for (const line of porcelain.split('\n')) {
    if (line.startsWith('worktree ')) {
      current = { path: line.slice(9), branch: null, head: null, bare: false }
      worktrees.push(current)
    } else if (current && line.startsWith('HEAD ')) current.head = line.slice(5)
    else if (current && line.startsWith('branch ')) {
      current.branch = line.slice(7).replace(/^refs\/heads\//, '')
    } else if (current && line === 'bare') current.bare = true
  }
  return worktrees
}

export const listWorktrees = async (cwd) =>
  parseWorktrees(await gitText(['worktree', 'list', '--porcelain'], { cwd }))

export const currentBranch = async (cwd) => {
  const { ok, stdout } = await git(['symbolic-ref', '--quiet', '--short', 'HEAD'], {
    cwd,
    allowFailure: true,
  })
  return ok ? stdout.trim() : null
}
