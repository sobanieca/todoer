import { component } from 'imp'
import { useHttpClient } from 'imp/http'
import { computed, initState, useState } from 'imp/state'
import { heading, link, text } from 'imp/std'
import { table } from 'imp/std/data'
import { emptyState, tag } from 'imp/std/display'
import { segmented } from 'imp/std/inputs'
import { flex, stack } from 'imp/std/layout'
import { app } from '../app-map.js'
import { formatTime, PR_COLORS, VERDICT_COLORS, VERDICT_LABELS } from '../lib/api.js'
import { page } from '../parts/page.js'

const FILTERS = [
  { value: 'open', label: 'Open' },
  { value: 'merged', label: 'Merged' },
  { value: 'closed', label: 'Closed' },
  { value: 'all', label: 'All' },
]

const rowOf = (pr) => ({
  id: pr.id,
  title: pr.title,
  status: pr.status,
  revision: pr.revision,
  verdict: pr.verdict,
  comments: pr.comments,
  todos: pr.todos.join(', '),
  updated: pr.updated ?? '',
})

const prHref = (id) => app.prs.$prId.href({ prId: id })

export const prsScreen = component('todoer-prs', {
  loading: () => text('Loading the pull requests...'),
  setup: async (self) => {
    const api = useHttpClient(self, 'todoer.api')
    const first = await api.get('/prs')
    if (self.abortSignal.aborted) return null
    const revision = useState(self, 'todoer.revision')
    const all = initState(self, 'todoer.prs.all', first.prs)
    const filter = initState(self, 'todoer.prs.filter', 'open')
    revision.watch(self, async () => {
      const { prs } = await api.get('/prs')
      if (!self.abortSignal.aborted) all.set(prs)
    })
    const rows = computed(
      [all, filter],
      (prs, chosen) => prs.filter((pr) => chosen === 'all' || pr.status === chosen).map(rowOf),
    )
    return page(
      heading('Pull requests'),
      all.view((prs) =>
        prs.length
          ? stack(
            { gap: 4 },
            flex(segmented({ label: 'Status', value: filter, options: FILTERS, size: 'sm' })),
            table({
              label: 'Pull requests',
              rows,
              columns: [
                { key: 'id', label: 'Id', render: (row) => link({ href: prHref(row.id) }, row.id) },
                {
                  key: 'title',
                  label: 'Title',
                  sortable: true,
                  render: (row) => link({ href: prHref(row.id) }, row.title),
                },
                {
                  key: 'status',
                  label: 'Status',
                  render: (row) => tag({ color: PR_COLORS[row.status], size: 'sm' }, row.status),
                },
                {
                  key: 'verdict',
                  label: 'Review',
                  render: (row) =>
                    tag(
                      { color: VERDICT_COLORS[row.verdict], size: 'sm' },
                      VERDICT_LABELS[row.verdict],
                    ),
                },
                {
                  key: 'revision',
                  label: 'Revision',
                  align: 'end',
                  render: (row) => `v${row.revision}`,
                },
                { key: 'comments', label: 'Comments', align: 'end' },
                { key: 'todos', label: 'Todos' },
                {
                  key: 'updated',
                  label: 'Updated',
                  sortable: true,
                  render: (row) => formatTime(row.updated),
                },
              ],
            }),
          )
          : emptyState({
            title: 'No pull requests yet',
            description: 'Select todos, click "Ask agent", and give the prompt to your agent. ' +
              'Its pull requests show here.',
          })
      ),
    )
  },
})
