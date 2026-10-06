import { requireOnlyFlags } from '../args.js'
import { reviewPrompt } from '../agent.js'
import { assertNotBlocked, blockersOf, describe, loadGraph } from '../dependencies.js'
import { logger } from '../logger.js'
import { findProject } from '../project.js'
import { addComment, closePr, listPrs, mergePr, readPrDetails, submitPr } from '../prs.js'

const printJson = (value) => console.log(JSON.stringify(value, null, 2))

const KIND_LABELS = { line: 'line', file: 'file', folder: 'folder' }

const submit = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['title', 'message', 'json'], 'pr submit')
  if (positional.length > 2) throw new Error('use: todoer pr submit [--title <t>] [--message <m>]')
  const project = await findProject()
  const result = await submitPr(project, { title: flags.title, message: flags.message })
  if (flags.json) return printJson({ ...result.pr, created: result.created })
  const { pr } = result
  logger.plain(
    [
      result.created
        ? `Created pull request ${pr.id}: ${pr.title}`
        : `Added revision ${pr.revision} to pull request ${pr.id}: ${pr.title}`,
      `${result.commits} commit(s) from ${pr.branch} on top of ${pr.base.slice(0, 10)}`,
      pr.todos.length ? `Todos: ${pr.todos.join(', ')}` : 'Todos: none claimed on this branch',
      '',
      'The person reviews it with: todoer ui',
    ].join('\n'),
  )
}

const list = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['all', 'json'], 'pr ls')
  if (positional.length > 2) throw new Error('use: todoer pr ls [--all] [--json]')
  const project = await findProject()
  const prs = (await listPrs(project)).filter((pr) => flags.all || pr.status === 'open')
  if (flags.json) return printJson(prs)
  if (!prs.length) return logger.plain('No open pull requests.')
  for (const pr of prs) {
    logger.plain(
      `${pr.id}  ${pr.status.padEnd(6)}  v${pr.revision}  ${pr.verdict.padEnd(17)}  ${pr.title}`,
    )
  }
}

const commentLines = (comment) => {
  const lines = [`[${comment.id}] ${comment.target}  (${KIND_LABELS[comment.kind]})`]
  if (comment.quote !== null && comment.quote !== undefined) lines.push(`  > ${comment.quote}`)
  for (const line of comment.text.split('\n')) lines.push(`  ${line}`)
  return lines
}

const show = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['revision', 'diff', 'json'], 'pr show')
  const [, , id] = positional
  if (!id || positional.length > 3) throw new Error('use: todoer pr show <id> [--diff] [--json]')
  const project = await findProject()
  const details = await readPrDetails(project, id, flags.revision)
  const { pr, review } = details
  const nodes = await loadGraph(project)
  const blockedBy = blockersOf(nodes, `pr:${pr.id}`)
  if (flags.json) {
    const { diff, files, ...rest } = details
    return printJson({
      ...rest,
      files: files.map(({ hunks: _hunks, ...file }) => file),
      blockedBy,
      ...(flags.diff ? { diff } : {}),
    })
  }
  const lines = [
    `${pr.id}  ${pr.title}`,
    `status: ${pr.status}   revision: ${details.revision} of ${pr.revision}   review: ${review.verdict}`,
    `branch: ${pr.branch}`,
    `worktree: ${details.worktreePath ?? '-'}${details.worktreeExists ? '' : ' (missing)'}`,
    `todos: ${pr.todos.join(', ') || '-'}`,
    `patch: ${details.patchFile}`,
    ...pr.dependsOn.map((entry) => `depends on: ${describe(nodes, entry)}`),
    blockedBy.length
      ? `BLOCKED: merge after ${blockedBy.map((entry) => describe(nodes, entry)).join(', ')}`
      : null,
    '',
    'Files:',
    ...details.files.map((file) =>
      `  ${file.status.padEnd(8)} ${
        file.oldPath && file.oldPath !== file.path ? `${file.oldPath} -> ` : ''
      }${file.path}  +${file.additions} -${file.deletions}`
    ),
  ]
  if (pr.description) lines.push('', 'Description:', pr.description)
  if (review.summary) lines.push('', 'Review summary:', review.summary)
  lines.push('', review.comments.length ? 'Comments:' : 'No review comments.')
  for (const comment of review.comments) lines.push(...commentLines(comment))
  for (const old of details.previous) {
    lines.push('', `Comments on revision ${old.revision} (${old.verdict}):`)
    for (const comment of old.comments) lines.push(...commentLines(comment))
  }
  if (flags.diff) lines.push('', details.diff)
  logger.plain(lines.filter((line) => line !== null).join('\n'))
}

const comment = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'pr comment')
  const [, , id, target, ...words] = positional
  if (!id || !target || !words.length) {
    throw new Error('use: todoer pr comment <id> <path:line | path | folder/> "<text>"')
  }
  const project = await findProject()
  const added = await addComment(project, id, { target, text: words.join(' ') })
  logger.plain(`Added comment ${added.id} on ${added.target}`)
}

const merge = async ({ positional, flags }) => {
  requireOnlyFlags(flags, ['keep', 'force'], 'pr merge')
  const [, , id] = positional
  if (!id || positional.length > 3) throw new Error('use: todoer pr merge <id> [--keep] [--force]')
  const project = await findProject()
  if (!flags.force) await assertNotBlocked(project, [`pr:${id}`], `merge pull request ${id}`)
  const { pr, warnings } = await mergePr(project, id, { keep: Boolean(flags.keep) })
  logger.plain(
    `Merged pull request ${pr.id} into ${project.mainBranch} (${pr.merged.slice(0, 10)})`,
  )
  for (const warning of warnings) logger.warn(warning)
}

const close = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'pr close')
  const [, , id] = positional
  if (!id || positional.length > 3) throw new Error('use: todoer pr close <id>')
  const project = await findProject()
  const { pr } = await closePr(project, id)
  logger.plain(`Closed pull request ${pr.id}. Its todos are open again.`)
}

const prompt = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'pr prompt')
  const [, , id] = positional
  if (!id || positional.length > 3) throw new Error('use: todoer pr prompt <id>')
  const project = await findProject()
  await readPrDetails(project, id)
  logger.plain(reviewPrompt(id))
}

const SUBCOMMANDS = { submit, ls: list, show, comment, merge, close, prompt }

export const pr = async (parsed) => {
  const [, name] = parsed.positional
  if (!name) throw new Error(`use: todoer pr <${Object.keys(SUBCOMMANDS).join('|')}>`)
  if (!Object.hasOwn(SUBCOMMANDS, name)) {
    throw new Error(`unknown command "pr ${name}" - run todoer help`)
  }
  await SUBCOMMANDS[name](parsed)
}
