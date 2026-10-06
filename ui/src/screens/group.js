import { component } from 'imp'
import { useHttpClient } from 'imp/http'
import { navigate } from 'imp/router'
import { computed, initState, useState } from 'imp/state'
import { button, heading, icon, link, menuButton, text } from 'imp/std'
import { table } from 'imp/std/data'
import { card, emptyState, tag } from 'imp/std/display'
import { notice, notify, progress } from 'imp/std/feedback'
import { textField } from 'imp/std/inputs'
import { dialog, flex, stack } from 'imp/std/layout'
import { luBot } from 'imp/icons'
import { app } from '../app-map.js'
import { messageOf, PR_COLORS } from '../lib/api.js'
import { groupNames, groupRoots, groupSlug, nodeHref, nodeMap } from '../lib/graph.js'
import { agentDialog } from '../parts/agent-dialog.js'
import { depTree, statusTag } from '../parts/dep-tree.js'
import { page } from '../parts/page.js'
import { groupHref } from './groups.js'

const summaryOf = (map, name) => {
  const todos = [...map.values()].filter((node) =>
    node.kind === 'todo' && node.groups.includes(name)
  )
  const prKeys = [...new Set(todos.map((todo) => todo.pr).filter(Boolean))]
  return {
    name,
    todos,
    prs: prKeys.map((key) => map.get(key)).filter(Boolean),
    done: todos.filter((todo) => todo.complete).length,
    ready: todos.filter((todo) => todo.ready).length,
    waiting: todos.filter((todo) => !todo.complete && todo.blockedBy.length).length,
  }
}

const rowOf = (todo) => ({
  id: todo.id,
  title: todo.title,
  status: todo.status,
  waits: todo.complete ? -1 : todo.status !== 'todo' ? -2 : todo.blockedBy.length,
  pr: todo.pr ? todo.pr.split(':')[1] : '',
})

const renameForm = component('todoer-rename-group', {
  setup: (self, { name, onDone }) => {
    const api = useHttpClient(self, 'todoer.api')
    const revision = useState(self, 'todoer.revision')
    const value = initState(self, 'todoer.rename.value', name)
    const failure = initState(self, 'todoer.rename.failure', null)
    const save = async () => {
      try {
        await api.post('/groups/rename', { body: { from: name, to: value.get() } })
        revision.set((current) => current + 1)
        onDone()
        navigate(groupHref(value.get().replace(/\s+/g, ' ').trim()), { replace: true })
      } catch (error) {
        failure.set(messageOf(error))
      }
    }
    return stack(
      { gap: 3 },
      textField({ label: 'Name', value, maxLength: 100 }),
      failure.view((message) => message && notice({ kind: 'error' }, message)),
      flex(
        { horizontalAlign: 'end' },
        button({ events: { click: onDone } }, 'Cancel'),
        button({ variant: 'primary', events: { click: save } }, 'Rename'),
      ),
    )
  },
})

export const groupScreen = component('todoer-group', {
  setup: (self, { slug }) => {
    const api = useHttpClient(self, 'todoer.api')
    const graph = useState(self, 'todoer.graph')
    const revision = useState(self, 'todoer.revision')
    const selected = initState(self, 'todoer.group.selected', [])
    const asking = initState(self, 'todoer.group.asking', false)
    const renaming = initState(self, 'todoer.group.renaming', false)
    const askFor = initState(self, 'todoer.group.askFor', [])
    const name = computed(
      [graph],
      (nodes) => groupNames(nodes).find((entry) => groupSlug(entry) === slug) ?? null,
    )
    const summary = computed(
      [graph, name],
      (nodes, current) => current ? summaryOf(nodeMap(nodes), current) : null,
    )
    const rows = computed([summary], (value) => (value ? value.todos.map(rowOf) : []))
    const ask = (ids) => {
      askFor.set(ids)
      asking.set(true)
    }
    const removeFromGroup = async () => {
      try {
        await api.post('/groups/remove-todos', {
          body: { name: name.get(), todos: selected.get() },
        })
        selected.set([])
        revision.set((value) => value + 1)
      } catch (error) {
        notify({
          kind: 'error',
          title: 'The todos stay in the group',
          description: messageOf(error),
        })
      }
    }
    const removeGroup = async () => {
      if (!confirm(`Remove the group "${name.get()}"? Its todos stay.`)) return
      await api.post('/groups/remove', { body: { name: name.get() } })
      revision.set((value) => value + 1)
      navigate(app.groups.href())
    }
    const noSelection = computed([selected], (ids) => ids.length === 0)
    return name.view((current) =>
      !current
        ? page(
          emptyState(
            { title: 'No such group', description: 'No todo is in this group now.' },
            link({ href: app.groups.href() }, 'All groups'),
          ),
        )
        : page(
          flex(
            { horizontalAlign: 'justify', verticalAlign: 'top', wrap: true },
            stack(
              { gap: 2 },
              heading(current),
              summary.view((value) =>
                flex(
                  { gap: 2, wrap: true, verticalAlign: 'center' },
                  tag({ color: 'green', size: 'sm' }, `${value.done} done`),
                  tag({ color: 'teal', size: 'sm' }, `${value.ready} ready`),
                  value.waiting
                    ? tag({ color: 'red', size: 'sm' }, `${value.waiting} waiting`)
                    : null,
                  tag({ size: 'sm' }, `${value.todos.length} todos`),
                  value.prs.length
                    ? tag({ color: 'blue', size: 'sm' }, `${value.prs.length} PRs`)
                    : null,
                )
              ),
            ),
            flex(
              { gap: 2 },
              button(
                {
                  events: {
                    click: () =>
                      ask(summary.get().todos.filter((todo) => todo.ready).map((todo) => todo.id)),
                  },
                  disabled: computed([summary], (value) => !value?.ready),
                },
                icon(luBot, { size: 'sm' }),
                'Ask agent for the ready todos',
              ),
              menuButton({
                label: 'Group actions',
                items: [
                  { id: 'rename', label: 'Rename the group', onClick: () => renaming.set(true) },
                  { id: 'remove', label: 'Remove the group', onClick: removeGroup },
                ],
              }),
            ),
          ),
          summary.view((value) =>
            progress({
              label: `${value.done} of ${value.todos.length} todos done`,
              value: value.done,
              max: Math.max(value.todos.length, 1),
              showValue: true,
              size: 'lg',
              color: value.done === value.todos.length ? 'success' : 'primary',
            })
          ),
          card(
            {
              header: flex(
                { horizontalAlign: 'justify', verticalAlign: 'center', wrap: true },
                heading({ level: 2 }, 'Todos'),
                flex(
                  { gap: 2 },
                  button(
                    {
                      size: 'sm',
                      disabled: noSelection,
                      events: { click: () => ask(selected.get()) },
                    },
                    'Ask agent',
                  ),
                  button(
                    { size: 'sm', disabled: noSelection, events: { click: removeFromGroup } },
                    'Remove from group',
                  ),
                ),
              ),
            },
            table({
              label: `Todos of ${current}`,
              rows,
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
                  render: (row) => statusTag({ kind: 'todo', status: row.status }),
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
              ],
            }),
          ),
          card(
            { header: heading({ level: 2 }, 'Order of work') },
            stack(
              { gap: 3 },
              text(
                { size: 'sm', tone: 'secondary' },
                'The top rows are the todos that no other todo of the group needs. Open a row to ' +
                  'see what it waits for, to any depth. The tree also shows items outside the group.',
              ),
              depTree({
                graph,
                roots: (map) =>
                  groupRoots(
                    map,
                    [...map.values()]
                      .filter((node) => node.kind === 'todo' && node.groups.includes(current))
                      .map((node) => node.key),
                  ),
                direction: 'dependsOn',
                label: `The order of work of ${current}`,
                empty: 'No todo in this group.',
              }),
            ),
          ),
          summary.view((value) =>
            value.prs.length
              ? card(
                { header: heading({ level: 2 }, 'Pull requests') },
                stack(
                  { gap: 2 },
                  ...value.prs.map((pr) =>
                    flex(
                      { gap: 2, verticalAlign: 'center' },
                      link({ href: nodeHref(pr) }, `${pr.id} - ${pr.title}`),
                      tag({ color: PR_COLORS[pr.status], size: 'sm' }, pr.status),
                    )
                  ),
                ),
              )
              : null
          ),
          agentDialog({ open: asking, request: () => ({ todos: askFor.get() }) }),
          dialog(
            { title: 'Rename the group', open: renaming },
            renaming.view((open) =>
              open ? renameForm({ name: current, onDone: () => renaming.set(false) }) : null
            ),
          ),
        )
    )
  },
})
