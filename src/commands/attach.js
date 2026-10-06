import { requireOnlyFlags } from '../args.js'
import { addImage, formatKb } from '../images.js'
import { logger } from '../logger.js'
import { findProject } from '../project.js'
import { getTodo, updateTodo } from '../todos.js'

export const attach = async ({ positional, flags }) => {
  requireOnlyFlags(flags, [], 'attach')
  const [, id, file] = positional
  if (!id || !file || positional.length > 3) throw new Error('use: todoer attach <id> <image>')
  const project = await findProject()
  const bytes = await Deno.readFile(file)
  const name = file.slice(file.lastIndexOf('/') + 1)
  const image = await addImage(project, id, { name, bytes })
  const todo = await getTodo(project, id)
  const body = todo.body.trim() ? `${todo.body.trim()}\n\n${image.markdown}` : image.markdown
  await updateTodo(project, id, { body })
  logger.plain(`Attached ${image.name} (${formatKb(image.size)}) to todo ${id}`)
}
