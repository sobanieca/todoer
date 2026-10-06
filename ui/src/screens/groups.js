import { component } from 'imp'
import { useHttpClient } from 'imp/http'
import { initState, useState } from 'imp/state'
import { heading, link, text } from 'imp/std'
import { table } from 'imp/std/data'
import { emptyState, tag } from 'imp/std/display'
import { progress } from 'imp/std/feedback'
import { textField } from 'imp/std/inputs'
import { flex, stack } from 'imp/std/layout'
import { app } from '../app-map.js'
import { groupSlug } from '../lib/graph.js'
import { page } from '../parts/page.js'

const rowOf = (group) => ({
  id: group.name,
  name: group.name,
  percent: group.percent,
  done: group.done,
  total: group.total,
  ready: group.ready,
  blocked: group.blocked,
  active: group.inProgress + group.review,
  openPrs: group.openPrs,
})

export const groupHref = (name) => app.groups.$group.href({ group: groupSlug(name) })

export const groupsScreen = component('todoer-groups', {
  loading: () => text('Loading the groups...'),
  setup: async (self) => {
    const api = useHttpClient(self, 'todoer.api')
    const first = await api.get('/groups')
    if (self.abortSignal.aborted) return null
    const revision = useState(self, 'todoer.revision')
    const rows = initState(self, 'todoer.groups.rows', first.groups.map(rowOf))
    const search = initState(self, 'todoer.groups.search', '')
    revision.watch(self, async () => {
      const { groups } = await api.get('/groups')
      if (!self.abortSignal.aborted) rows.set(groups.map(rowOf))
    })
    return page(
      heading('Groups'),
      rows.view((list) =>
        list.length
          ? stack(
            { gap: 4 },
            flex(
              textField({
                ariaLabel: 'Search the groups',
                type: 'search',
                value: search,
                placeholder: 'Search',
              }),
            ),
            table({
              label: 'Groups',
              rows,
              filter: search,
              columns: [
                {
                  key: 'name',
                  label: 'Group',
                  sortable: true,
                  render: (row) => link({ href: groupHref(row.name) }, row.name),
                },
                {
                  key: 'percent',
                  label: 'Progress',
                  sortable: true,
                  render: (row) =>
                    progress({
                      label: `${row.done} of ${row.total} done`,
                      value: row.done,
                      max: row.total,
                      showValue: true,
                      color: row.done === row.total ? 'success' : 'primary',
                    }),
                },
                { key: 'ready', label: 'Ready', align: 'end', sortable: true },
                { key: 'active', label: 'In progress', align: 'end', sortable: true },
                {
                  key: 'blocked',
                  label: 'Waiting',
                  align: 'end',
                  sortable: true,
                  render: (row) =>
                    row.blocked ? tag({ color: 'red', size: 'sm' }, row.blocked) : '0',
                },
                { key: 'openPrs', label: 'Open PRs', align: 'end', sortable: true },
              ],
            }),
          )
          : emptyState({
            title: 'No groups yet',
            description: 'Select todos on the Todos page and click "Add to group", or set the ' +
              'groups of one todo on its page.',
          })
      ),
    )
  },
})
