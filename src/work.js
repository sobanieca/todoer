import { assertNotBlocked } from './dependencies.js'
import { git } from './git.js'
import { exists } from './fs-utils.js'
import {
  absoluteFromRoot,
  branchOf,
  ensureDataDir,
  excludeFromGit,
  relativeToRoot,
} from './project.js'
import { getTodo, updateTodo } from './todos.js'
import { join } from './paths.js'

const checkTodos = async (project, ids) => {
  if (!ids.length) throw new Error('name one or more todo ids, like: todoer start 7f3a 9b2c')
  const todos = []
  for (const id of new Set(ids)) {
    const todo = await getTodo(project, id)
    if (todo.status === 'done') throw new Error(`todo ${id} is done`)
    if (todo.worktree && await exists(absoluteFromRoot(project, todo.worktree))) {
      throw new Error(`todo ${id} is in progress in ${absoluteFromRoot(project, todo.worktree)}`)
    }
    todos.push(todo)
  }
  return todos
}

export const startWork = async (project, ids, { here = false, force = false } = {}) => {
  const todos = await checkTodos(project, ids)
  if (!force) {
    await assertNotBlocked(project, todos.map((todo) => `todo:${todo.id}`), 'start the work')
  }
  await ensureDataDir(project)
  let path
  let branch
  if (here) {
    if (!project.inWorktree) {
      throw new Error('--here works in a linked worktree only - never work on the main checkout')
    }
    path = project.current
    branch = await branchOf(path)
    if (!branch) throw new Error('the worktree has no branch checked out')
    if (branch === project.mainBranch) {
      throw new Error(`the worktree is on ${project.mainBranch} - use a branch of its own`)
    }
  } else {
    const name = todos.map((todo) => todo.id).join('-')
    branch = `todoer/${name}`
    path = join(project.root, project.config.worktreesDir, name)
    if (await exists(path)) throw new Error(`${path} exists - remove it, or use --here in it`)
    if (absoluteFromRoot(project, project.config.worktreesDir).startsWith(`${project.root}/`)) {
      await excludeFromGit(project, `/${project.config.worktreesDir.replace(/\/+$/, '')}/`)
    }
    await git(['worktree', 'add', '-b', branch, path, project.mainBranch], { cwd: project.root })
  }
  const worktree = relativeToRoot(project, path)
  const updated = []
  for (const todo of todos) {
    updated.push(await updateTodo(project, todo.id, { status: 'in-progress', branch, worktree }))
  }
  return { path, branch, todos: updated }
}
