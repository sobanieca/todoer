import { ensureDataDir, projectPath } from './project.js'
import { now } from './ids.js'
import { join } from './paths.js'

export const implementPrompt = (ids) =>
  ids.length === 1
    ? `Implement the todoer todo ${ids[0]}. Run "todoer skill" and follow the workflow it prints.`
    : `Implement the todoer todos ${ids.join(', ')} in one worktree, as one pull request. ` +
      'Run "todoer skill" and follow the workflow it prints.'

export const reviewPrompt = (prId) =>
  `Address the review of the todoer pull request ${prId}. ` +
  'Run "todoer skill" and follow the workflow it prints.'

export const agentCommandFor = (command, prompt) =>
  command.includes('{prompt}')
    ? command.map((part) => part.replaceAll('{prompt}', prompt))
    : [...command, prompt]

const shellQuote = (value) => `'${value.replaceAll("'", "'\\''")}'`

export const runAgent = async (project, prompt) => {
  const command = project.config.agentCommand
  if (!command) {
    throw new Error(
      'set "agentCommand" in todoer.json to run an agent, like ["claude", "{prompt}"]',
    )
  }
  await ensureDataDir(project)
  const runsDir = projectPath(project, 'runs')
  await Deno.mkdir(runsDir, { recursive: true })
  const log = join(runsDir, `${now().replace(/[:]/g, '-')}.log`)
  const argv = agentCommandFor(command, prompt)
  const script = `exec ${argv.map(shellQuote).join(' ')} > ${shellQuote(log)} 2>&1 < /dev/null`
  const child = new Deno.Command('sh', {
    args: ['-c', script],
    cwd: project.root,
    stdin: 'null',
    stdout: 'null',
    stderr: 'null',
  }).spawn()
  child.unref()
  return { pid: child.pid, log, command: argv }
}
