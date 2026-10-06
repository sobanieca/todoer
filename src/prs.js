import { lineText, parseDiff } from './diff.js'
import { formatDocument, listField, parseDocument } from './frontmatter.js'
import { git, gitText } from './git.js'
import { exists, listDirectory, readTextIfExists, writeFileAtomic } from './fs-utils.js'
import { ID_PATTERN, now, uniqueId } from './ids.js'
import {
  absoluteFromRoot,
  branchOf,
  ensureDataDir,
  projectPath,
  relativeToRoot,
} from './project.js'
import {
  cleanCommentText,
  formatTarget,
  newCommentId,
  parseTarget,
  readReview,
  VERDICTS,
  writeReview,
} from './reviews.js'
import { listTodos, NotFoundError, takenIds, updateTodo } from './todos.js'
import { join } from './paths.js'

export const PR_STATUSES = ['open', 'merged', 'closed']

const FIELDS = [
  'id',
  'title',
  'status',
  'branch',
  'worktree',
  'base',
  'todos',
  'dependsOn',
  'revision',
  'created',
  'updated',
  'merged',
]

const EXCLUDE_DATA = ['--', '.', ':(exclude).todoer']

const prsDir = (project) => projectPath(project, 'prs')

const prDir = (project, id) => join(prsDir(project), id)

const patchPath = (project, id, revision) => join(prDir(project, id), `v${revision}.patch`)

const diffPath = (project, id, revision) => join(prDir(project, id), `v${revision}.diff`)

const reviewPath = (project, id, revision) => join(prDir(project, id), `review.v${revision}.md`)

const toPr = (text) => {
  const { fields, body } = parseDocument(text)
  return {
    id: fields.id,
    title: fields.title ?? fields.id,
    status: PR_STATUSES.includes(fields.status) ? fields.status : 'open',
    branch: fields.branch || null,
    worktree: fields.worktree || null,
    base: fields.base || null,
    todos: listField(fields.todos),
    dependsOn: listField(fields.dependsOn),
    revision: Number(fields.revision) || 1,
    created: fields.created ?? null,
    updated: fields.updated ?? fields.created ?? null,
    merged: fields.merged || null,
    description: body.trim(),
  }
}

export const setPrDependsOn = async (project, id, dependsOn) => {
  const pr = await getPr(project, id)
  const next = { ...pr, dependsOn: [...new Set(dependsOn)], updated: now() }
  await savePr(project, next)
  return next
}

const savePr = (project, pr) =>
  writeFileAtomic(
    join(prDir(project, pr.id), 'pr.md'),
    formatDocument(Object.fromEntries(FIELDS.map((key) => [key, pr[key]])), pr.description),
  )

export const getPr = async (project, id) => {
  if (!ID_PATTERN.test(id ?? '')) throw new NotFoundError(`"${id}" is not a pull request id`)
  const text = await readTextIfExists(join(prDir(project, id), 'pr.md'))
  if (text === null) throw new NotFoundError(`there is no pull request ${id} - run todoer pr ls`)
  return toPr(text)
}

export const listPrs = async (project) => {
  const prs = []
  for (const entry of await listDirectory(prsDir(project))) {
    if (!entry.isDirectory || !ID_PATTERN.test(entry.name)) continue
    const text = await readTextIfExists(join(prDir(project, entry.name), 'pr.md'))
    if (text === null) continue
    const pr = toPr(text)
    const review = await readReview(reviewPath(project, pr.id, pr.revision), pr.revision)
    prs.push({ ...pr, verdict: review.verdict, comments: review.comments.length })
  }
  return prs.sort((a, b) => (b.updated ?? '').localeCompare(a.updated ?? ''))
}

const commitSubjects = async (cwd, base) =>
  (await gitText(['log', '--reverse', '--format=%s', `${base}..HEAD`], { cwd }))
    .split('\n')
    .filter(Boolean)

export const submitPr = async (project, { title, message } = {}) => {
  if (!project.inWorktree) {
    throw new Error(
      'run todoer pr submit in the worktree of the todos, not in the main checkout',
    )
  }
  const cwd = project.current
  const branch = await branchOf(cwd)
  if (!branch) throw new Error('the worktree has no branch checked out')
  if (branch === project.mainBranch) {
    throw new Error(`the worktree is on ${project.mainBranch} - work on a branch of its own`)
  }
  const dirty = await gitText(['status', '--porcelain', ...EXCLUDE_DATA], { cwd })
  if (dirty) throw new Error(`the worktree has changes that are not committed:\n${dirty}`)
  const base = await gitText(['merge-base', 'HEAD', project.mainBranch], { cwd })
  const subjects = await commitSubjects(cwd, base)
  if (!subjects.length) {
    throw new Error(`there is no commit on ${branch} after ${project.mainBranch}`)
  }
  const patch = (await git(['format-patch', '--stdout', `${base}..HEAD`, ...EXCLUDE_DATA], { cwd }))
    .stdout
  const diff = (await git(['diff', '-M', base, 'HEAD', ...EXCLUDE_DATA], { cwd })).stdout
  if (!diff.trim()) throw new Error(`the commits on ${branch} change no file outside .todoer/`)
  await ensureDataDir(project)
  const todos = (await listTodos(project))
    .filter((todo) => todo.branch === branch)
    .sort((a, b) => (a.created ?? '').localeCompare(b.created ?? '') || a.id.localeCompare(b.id))
  const existing = (await listPrs(project)).find((pr) =>
    pr.branch === branch && pr.status === 'open'
  )
  const stamp = now()
  let pr
  if (existing) {
    pr = {
      ...(await getPr(project, existing.id)),
      revision: existing.revision + 1,
      base,
      worktree: relativeToRoot(project, cwd),
      todos: [...new Set([...existing.todos, ...todos.map((todo) => todo.id)])],
      updated: stamp,
    }
  } else {
    const taken = await takenIds(project)
    pr = {
      id: uniqueId(taken),
      title: todos.length === 1 ? todos[0].title : subjects[0],
      status: 'open',
      branch,
      worktree: relativeToRoot(project, cwd),
      base,
      todos: todos.map((todo) => todo.id),
      dependsOn: [],
      revision: 1,
      created: stamp,
      updated: stamp,
      merged: null,
      description: '',
    }
  }
  if (title?.trim()) pr.title = title.replace(/\s+/g, ' ').trim()
  if (message?.trim()) {
    pr.description = existing && pr.description
      ? `${pr.description}\n\n## Revision ${pr.revision}\n\n${message.trim()}`
      : message.trim()
  }
  await writeFileAtomic(patchPath(project, pr.id, pr.revision), patch)
  await writeFileAtomic(diffPath(project, pr.id, pr.revision), diff)
  await savePr(project, pr)
  for (const id of pr.todos) {
    await updateTodo(project, id, { status: 'review', pr: pr.id }).catch(() => {})
  }
  return { pr, created: !existing, commits: subjects.length }
}

export const readPrDetails = async (project, id, revision) => {
  const pr = await getPr(project, id)
  const chosen = revision === undefined || revision === null ? pr.revision : Number(revision)
  if (!Number.isInteger(chosen) || chosen < 1 || chosen > pr.revision) {
    throw new NotFoundError(`pull request ${id} has revisions 1 to ${pr.revision}`)
  }
  const diff = (await readTextIfExists(diffPath(project, id, chosen))) ?? ''
  const files = parseDiff(diff)
  const review = await readReview(reviewPath(project, id, chosen), chosen)
  const previous = []
  for (let number = 1; number < chosen; number++) {
    const old = await readReview(reviewPath(project, id, number), number)
    if (old.comments.length || old.summary) previous.push(old)
  }
  const worktreePath = pr.worktree ? absoluteFromRoot(project, pr.worktree) : null
  return {
    pr,
    revision: chosen,
    revisions: Array.from({ length: pr.revision }, (_, index) => index + 1),
    files,
    diff,
    review,
    previous,
    patchFile: patchPath(project, id, chosen),
    worktreePath,
    worktreeExists: worktreePath ? await exists(worktreePath) : false,
  }
}

const requireOpen = (pr) => {
  if (pr.status !== 'open') throw new Error(`pull request ${pr.id} is ${pr.status}`)
}

const editReview = async (project, id, revision, change) => {
  const pr = await getPr(project, id)
  const number = revision === undefined || revision === null ? pr.revision : Number(revision)
  if (number !== pr.revision) throw new Error('only the latest revision takes new review comments')
  requireOpen(pr)
  const path = reviewPath(project, id, number)
  const review = await readReview(path, number)
  const result = await change(review, pr)
  await writeReview(path, review)
  return result
}

export const addComment = (project, id, { target, text, revision }) =>
  editReview(project, id, revision, async (review) => {
    const parsed = parseTarget(target)
    const comment = {
      id: newCommentId(review.comments),
      ...parsed,
      target: formatTarget(parsed),
      quote: null,
      text: cleanCommentText(text),
    }
    if (parsed.kind === 'line') {
      const diff = (await readTextIfExists(diffPath(project, id, review.revision))) ?? ''
      comment.quote = lineText(parseDiff(diff), parsed)
    }
    review.comments.push(comment)
    return comment
  })

export const updateComment = (project, id, commentId, { text }) =>
  editReview(project, id, undefined, (review) => {
    const comment = review.comments.find((entry) => entry.id === commentId)
    if (!comment) throw new NotFoundError(`there is no comment ${commentId}`)
    comment.text = cleanCommentText(text)
    return comment
  })

export const removeComment = (project, id, commentId) =>
  editReview(project, id, undefined, (review) => {
    const index = review.comments.findIndex((entry) => entry.id === commentId)
    if (index === -1) throw new NotFoundError(`there is no comment ${commentId}`)
    review.comments.splice(index, 1)
  })

export const setVerdict = async (project, id, { verdict, summary }) => {
  if (!VERDICTS.includes(verdict)) {
    throw new Error(`"${verdict}" is not a verdict - use ${VERDICTS.join(', ')}`)
  }
  const review = await editReview(project, id, undefined, (review) => {
    review.verdict = verdict
    if (summary !== undefined) review.summary = String(summary ?? '').trim()
    return review
  })
  if (verdict === 'changes-requested') {
    const pr = await getPr(project, id)
    for (const todoId of pr.todos) {
      await updateTodo(project, todoId, { status: 'in-progress' }).catch(() => {})
    }
  }
  return review
}

export const mergePr = async (project, id, { keep = false } = {}) => {
  const pr = await getPr(project, id)
  requireOpen(pr)
  const root = project.root
  const branch = await branchOf(root)
  if (branch !== project.mainBranch) {
    throw new Error(
      `check out ${project.mainBranch} in ${root} first - it is on ${branch ?? 'no branch'}`,
    )
  }
  const staged = await git(['diff', '--cached', '--quiet'], { cwd: root, allowFailure: true })
  if (!staged.ok) throw new Error(`${root} has staged changes - commit or unstage them first`)
  const applied = await git(['am', '--3way', patchPath(project, id, pr.revision)], {
    cwd: root,
    allowFailure: true,
  })
  if (!applied.ok) {
    await git(['am', '--abort'], { cwd: root, allowFailure: true })
    throw new Error(
      `the patch does not apply on ${project.mainBranch} - ask the agent to rebase the branch ` +
        `and submit again:\n${(applied.stderr || applied.stdout).trim()}`,
    )
  }
  const merged = await gitText(['rev-parse', 'HEAD'], { cwd: root })
  const warnings = []
  const next = { ...pr, status: 'merged', merged, updated: now() }
  await savePr(project, next)
  for (const todoId of pr.todos) {
    await updateTodo(project, todoId, { status: 'done', branch: null, worktree: null })
      .catch((error) => warnings.push(`todo ${todoId}: ${error.message}`))
  }
  if (!keep) warnings.push(...await removeWork(project, pr))
  return { pr: next, warnings }
}

const removeWork = async (project, pr) => {
  const warnings = []
  const path = pr.worktree ? absoluteFromRoot(project, pr.worktree) : null
  if (path && path !== project.root && await exists(path)) {
    const removed = await git(['worktree', 'remove', path], {
      cwd: project.root,
      allowFailure: true,
    })
    if (!removed.ok) warnings.push(`the worktree ${path} stays: ${removed.stderr.trim()}`)
  }
  if (pr.branch && pr.branch !== project.mainBranch) {
    const deleted = await git(['branch', '-D', pr.branch], {
      cwd: project.root,
      allowFailure: true,
    })
    if (!deleted.ok) warnings.push(`the branch ${pr.branch} stays: ${deleted.stderr.trim()}`)
  }
  return warnings
}

export const closePr = async (project, id, { keep = true } = {}) => {
  const pr = await getPr(project, id)
  requireOpen(pr)
  const next = { ...pr, status: 'closed', updated: now() }
  await savePr(project, next)
  for (const todoId of pr.todos) {
    await updateTodo(project, todoId, { status: 'todo', pr: null, branch: null, worktree: null })
      .catch(() => {})
  }
  const warnings = keep ? [] : await removeWork(project, pr)
  return { pr: next, warnings }
}
