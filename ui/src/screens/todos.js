import { component } from 'imp'
import { useHttpClient } from 'imp/http'
import { computed, initState, useState } from 'imp/state'
import { button, heading, icon, link, text } from 'imp/std'
import { table } from 'imp/std/data'
import { emptyState, tag } from 'imp/std/display'
import { notice, notify } from 'imp/std/feedback'
import {
  comboBox,
  fileUpload,
  form,
  FormError,
  segmented,
  select,
  submitForm,
  textField,
} from 'imp/std/inputs'
import { dialog, flex, stack } from 'imp/std/layout'
import { luBot, luFolderPlus, luPlus } from 'imp/icons'
import { app } from '../app-map.js'
import {
  appendImage,
  formatTime,
  messageOf,
  STATUS_COLORS,
  STATUS_LABELS,
  uploadImage,
} from '../lib/api.js'
import { agentDialog } from '../parts/agent-dialog.js'
import { groupNames, nodeMap } from '../lib/graph.js'
import { groupHref } from './groups.js'
import { page } from '../parts/page.js'

const FILTERS = [
  { value: 'open', label: 'Open' },
  { value: 'ready', label: 'Ready' },
  { value: 'todo', label: 'To do' },
  { value: 'in-progress', label: 'In progress' },
  { value: 'review', label: 'In review' },
  { value: 'done', label: 'Done' },
  { value: 'all', label: 'All' },
]

const matches = (filter, todo, node) => {
  if (filter === 'all') return true
  if (filter === 'open') return todo.status !== 'done'
  if (filter === 'ready') return Boolean(node?.ready)
  return todo.status === filter
}

const rowOf = (todo, node) => ({
  id: todo.id,
  title: todo.title,
  status: todo.status,
  groups: todo.groups,
  waits: todo.status === 'done' ? -1 : todo.status !== 'todo' ? -2 : node?.blockedBy.length ?? 0,
  pr: todo.pr ?? '',
  updated: todo.updated ?? '',
})

const ALL_GROUPS = '*'

const groupForm = component('todoer-add-to-group', {
  setup: (self, { ids, onDone }) => {
    const api = useHttpClient(self, 'todoer.api')
    const graph = useState(self, 'todoer.graph')
    const revision = useState(self, 'todoer.revision')
    const name = initState(self, 'todoer.addgroup.name', undefined)
    const failure = initState(self, 'todoer.addgroup.failure', null)
    const empty = computed([name], (value) => !value?.trim())
    const save = async () => {
      try {
        await api.post('/groups/add', { body: { name: name.get(), todos: ids } })
        revision.set((value) => value + 1)
        notify({ kind: 'success', title: `Added ${ids.length} todo(s) to "${name.get().trim()}"` })
        onDone()
      } catch (error) {
        failure.set(messageOf(error))
      }
    }
    return stack(
      { gap: 3 },
      comboBox({
        label: 'Group',
        value: name,
        options: groupNames(graph.get()),
        custom: true,
        placeholder: 'Pick a group, or type a new name',
        help: 'For example "tests refactoring" or "bump some-npm-package version to 0.3.5".',
      }),
      failure.view((message) => message && notice({ kind: 'error' }, message)),
      flex(
        { horizontalAlign: 'end' },
        button({ events: { click: onDone } }, 'Cancel'),
        button({ variant: 'primary', disabled: empty, events: { click: save } }, 'Add to group'),
      ),
    )
  },
})

const newTodoForm = component('todoer-new-todo', {
  setup: (self, { onDone }) => {
    const api = useHttpClient(self, 'todoer.api')
    const project = useState(self, 'todoer.project')
    const { maxImageBytes, maxImageKb } = project.get()
    let files = []
    return form(
      {
        initialValues: { title: '', body: '', images: [] },
        onSubmit: async (values) => {
          let todo
          try {
            todo = await api.post('/todos', { body: { title: values.title, body: values.body } })
          } catch (error) {
            throw new FormError(messageOf(error), { cause: error })
          }
          let body = todo.body
          const failed = []
          for (const file of files) {
            try {
              const image = await uploadImage(api, todo.id, file)
              body = appendImage(body, image.markdown)
            } catch (error) {
              failed.push(`${file.name}: ${messageOf(error)}`)
            }
          }
          if (body !== todo.body) await api.patch(`/todos/${todo.id}`, { body: { body } })
          notify({ kind: 'success', title: `Added todo ${todo.id}`, description: todo.title })
          if (failed.length) {
            notify({
              kind: 'warning',
              title: 'Some images were not added',
              description: failed.join(' '),
            })
          }
          onDone()
        },
      },
      textField({ field: 'title', label: 'Title', required: true, maxLength: 200 }),
      textField({
        field: 'body',
        label: 'Description',
        rows: 6,
        help: 'Markdown. Tell the agent what to change and how to check it.',
      }),
      fileUpload({
        field: 'images',
        label: 'Images',
        multiple: true,
        accept: 'image/png,image/jpeg,image/gif,image/webp',
        maxSize: maxImageBytes,
        help: `maxImageKb in todoer.json sets the limit of ${maxImageKb} kB.`,
        onFiles: (chosen) => {
          files = chosen
        },
      }),
      flex(
        { horizontalAlign: 'end' },
        button({ variant: 'primary', events: { click: submitForm } }, 'Add todo'),
      ),
    )
  },
})

export const todosScreen = component('todoer-todos', {
  loading: () => text('Loading the todos...'),
  setup: async (self) => {
    const api = useHttpClient(self, 'todoer.api')
    const revision = useState(self, 'todoer.revision')
    const first = await api.get('/todos')
    if (self.abortSignal.aborted) return null
    const all = initState(self, 'todoer.todos.all', first.todos)
    const filter = initState(self, 'todoer.todos.filter', 'open')
    const search = initState(self, 'todoer.todos.search', '')
    const selected = initState(self, 'todoer.todos.selected', [])
    const creating = initState(self, 'todoer.todos.creating', false)
    const asking = initState(self, 'todoer.todos.asking', false)
    const graph = useState(self, 'todoer.graph')
    const groupFilter = initState(self, 'todoer.todos.group', ALL_GROUPS)
    const grouping = initState(self, 'todoer.todos.grouping', false)
    const groupOptions = computed([graph], (nodes) => [
      { value: ALL_GROUPS, label: 'All groups' },
      ...groupNames(nodes).map((name) => ({ value: name, label: name })),
    ])
    const rows = computed([all, filter, graph, groupFilter], (todos, chosen, nodes, group) => {
      const map = nodeMap(nodes)
      return todos
        .filter((todo) => group === ALL_GROUPS || todo.groups.includes(group))
        .filter((todo) => matches(chosen, todo, map.get(`todo:${todo.id}`)))
        .map((todo) => rowOf(todo, map.get(`todo:${todo.id}`)))
    })
    const reload = async () => {
      const { todos } = await api.get('/todos')
      if (self.abortSignal.aborted) return
      all.set(todos)
      const ids = new Set(todos.map((todo) => todo.id))
      selected.set((current) => current.filter((id) => ids.has(id)))
    }
    revision.watch(self, reload)
    const noSelection = computed([selected], (ids) => ids.length === 0)
    const hasTodos = computed([all], (todos) => todos.length > 0)
    return page(
      flex(
        { horizontalAlign: 'justify', verticalAlign: 'center', wrap: true },
        heading('Todos'),
        flex(
          { gap: 2 },
          button(
            { disabled: noSelection, events: { click: () => grouping.set(true) } },
            icon(luFolderPlus, { size: 'sm' }),
            'Add to group',
          ),
          button(
            { disabled: noSelection, events: { click: () => asking.set(true) } },
            icon(luBot, { size: 'sm' }),
            computed([selected], (ids) => ids.length ? `Ask agent (${ids.length})` : 'Ask agent')
              .text(),
          ),
          button(
            { variant: 'primary', events: { click: () => creating.set(true) } },
            icon(luPlus, { size: 'sm' }),
            'New todo',
          ),
        ),
      ),
      hasTodos.view((any) =>
        any
          ? stack(
            { gap: 4 },
            flex(
              { wrap: true, verticalAlign: 'center' },
              segmented({ label: 'Status', value: filter, options: FILTERS, size: 'sm' }),
              groupOptions.view((options) =>
                select({ label: 'Group', value: groupFilter, options })
              ),
              textField({
                ariaLabel: 'Search the todos',
                type: 'search',
                value: search,
                placeholder: 'Search',
              }),
            ),
            table({
              label: 'Todos',
              rows,
              filter: search,
              selection: 'multiple',
              selected,
              columns: [
                {
                  key: 'id',
                  label: 'Id',
                  render: (row) =>
                    link({ href: app.todos.$todoId.href({ todoId: row.id }) }, row.id),
                },
                {
                  key: 'title',
                  label: 'Title',
                  sortable: true,
                  render: (row) =>
                    link({ href: app.todos.$todoId.href({ todoId: row.id }) }, row.title),
                },
                {
                  key: 'status',
                  label: 'Status',
                  sortable: true,
                  render: (row) =>
                    tag(
                      { color: STATUS_COLORS[row.status], size: 'sm' },
                      STATUS_LABELS[row.status],
                    ),
                },
                {
                  key: 'groups',
                  label: 'Groups',
                  sortable: true,
                  render: (row) =>
                    row.groups.length
                      ? flex(
                        { gap: 1, wrap: true },
                        ...row.groups.map((group) =>
                          link({ href: groupHref(group), size: 'sm' }, group)
                        ),
                      )
                      : '',
                },
                {
                  key: 'waits',
                  label: 'Can start',
                  sortable: true,
                  render: (row) =>
                    row.waits === -1
                      ? '-'
                      : row.waits === -2
                      ? tag({ color: 'blue', size: 'sm' }, 'started')
                      : row.waits
                      ? tag({ color: 'red', size: 'sm' }, `waits for ${row.waits}`)
                      : tag({ color: 'teal', size: 'sm' }, 'yes'),
                },
                {
                  key: 'pr',
                  label: 'Pull request',
                  render: (row) =>
                    row.pr ? link({ href: app.prs.$prId.href({ prId: row.pr }) }, row.pr) : '',
                },
                {
                  key: 'updated',
                  label: 'Updated',
                  sortable: true,
                  render: (row) => formatTime(row.updated),
                },
              ],
            }),
          )
          : emptyState(
            {
              title: 'No todos yet',
              description: 'Add a todo here, or run todoer add "<title>" in the project.',
            },
            button({ variant: 'primary', events: { click: () => creating.set(true) } }, 'New todo'),
          )
      ),
      dialog(
        { title: 'New todo', open: creating },
        creating.view((open) =>
          open
            ? newTodoForm({
              onDone: () => {
                creating.set(false)
                reload()
              },
            })
            : null
        ),
      ),
      agentDialog({ open: asking, request: () => ({ todos: selected.get() }) }),
      dialog(
        { title: 'Add to group', open: grouping },
        grouping.view((open) =>
          open ? groupForm({ ids: selected.get(), onDone: () => grouping.set(false) }) : null
        ),
      ),
    )
  },
})
