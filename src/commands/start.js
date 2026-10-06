import { requireOnlyFlags } from '../args.js'
import { logger } from '../logger.js'
import { findProject } from '../project.js'
import { startWork } from '../work.js'

export const start = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['here', 'force', 'json'], 'start')
  const ids = positional.slice(1)
  const project = await findProject()
  const result = await startWork(project, ids, {
    here: Boolean(flags.here),
    force: Boolean(flags.force),
  })
  if (flags.json) {
    return console.log(JSON.stringify({ path: result.path, branch: result.branch }, null, 2))
  }
  logger.plain(
    [
      `Claimed ${result.todos.map((todo) => todo.id).join(', ')} on the branch ${result.branch}.`,
      `Worktree: ${result.path}`,
      '',
      'Work and commit in the worktree, then run there:',
      '  todoer pr submit --title "<title>" --message "<summary>"',
    ].join('\n'),
  )
}
