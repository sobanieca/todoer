import { component } from 'imp'
import { useHttpClient } from 'imp/http'
import { computed, initState, useState } from 'imp/state'
import { button, heading, link, text } from 'imp/std'
import { card } from 'imp/std/display'
import { notice, notify } from 'imp/std/feedback'
import { comboBox } from 'imp/std/inputs'
import { dialog, flex, stack, tabs } from 'imp/std/layout'
import { list } from 'imp/std/data'
import { messageOf } from '../lib/api.js'
import { dependencyOptions, nodeHref, nodeLabel, nodeMap, refName } from '../lib/graph.js'
import { depTree, statusTag } from './dep-tree.js'

const addForm = component('todoer-add-dependency', {
  setup: (self, { itemKey, onDone }) => {
    const api = useHttpClient(self, 'todoer.api')
    const graph = useState(self, 'todoer.graph')
    const revision = useState(self, 'todoer.revision')
    const chosen = initState(self, 'todoer.dependency.chosen', [])
    const busy = initState(self, 'todoer.dependency.busy', false)
    const failure = initState(self, 'todoer.dependency.failure', null)
    const options = dependencyOptions(nodeMap(graph.get()), itemKey)
    const nothing = computed([chosen], (keys) => keys.length === 0)
    const add = async () => {
      busy.set(true)
      try {
        for (const key of chosen.get()) {
          await api.post('/dependencies', { body: { from: itemKey, to: key } })
        }
        revision.set((value) => value + 1)
        notify({ kind: 'success', title: `Added ${chosen.get().length} dependency(ies)` })
        onDone()
      } catch (error) {
        failure.set(messageOf(error))
        busy.set(false)
      }
    }
    return stack(
      { gap: 3 },
      text(
        { size: 'sm', tone: 'secondary' },
        `${
          refName(itemKey)
        } waits until each chosen todo is done and each chosen pull request is ` +
          'merged. The list leaves out every item that makes a cycle.',
      ),
      options.length
        ? comboBox({
          label: 'Depends on',
          value: chosen,
          options,
          multiple: true,
          placeholder: 'Type an id or a title',
        })
        : notice({ kind: 'info' }, 'No todo or pull request can be added without a cycle.'),
      failure.view((message) => message && notice({ kind: 'error' }, message)),
      flex(
        { horizontalAlign: 'end' },
        button({ events: { click: onDone } }, 'Cancel'),
        button(
          { variant: 'primary', disabled: nothing, loading: busy, events: { click: add } },
          'Add',
        ),
      ),
    )
  },
})

export const dependenciesCard = component('todoer-dependencies', {
  setup: (self, { itemKey }) => {
    const api = useHttpClient(self, 'todoer.api')
    const graph = useState(self, 'todoer.graph')
    const revision = useState(self, 'todoer.revision')
    const adding = initState(self, 'todoer.dependencies.adding', false)
    const failure = initState(self, 'todoer.dependencies.failure', null)
    const view = computed([graph], (nodes) => {
      const map = nodeMap(nodes)
      const node = map.get(itemKey)
      return {
        node,
        direct: (node?.dependsOn ?? []).map((key) => map.get(key) ?? { key, missing: true }),
        blockedBy: (node?.blockedBy ?? []).map((key) => map.get(key)).filter(Boolean),
      }
    })
    const remove = async (key) => {
      try {
        await api.post('/dependencies/remove', { body: { from: itemKey, to: key } })
        revision.set((value) => value + 1)
      } catch (error) {
        failure.set(messageOf(error))
      }
    }
    const directList = computed([view], ({ direct }) =>
      direct.map((node) => ({
        id: node.key,
        title: node.missing ? `${refName(node.key)} - missing` : nodeLabel(node),
        href: node.missing ? undefined : nodeHref(node),
      })))
    const hasDirect = computed([view], ({ direct }) => direct.length > 0)
    return card(
      {
        header: flex(
          { horizontalAlign: 'justify', verticalAlign: 'center' },
          heading({ level: 2 }, 'Dependencies'),
          button({ size: 'sm', events: { click: () => adding.set(true) } }, 'Add dependency'),
        ),
      },
      stack(
        { gap: 4 },
        view.view(({ node, blockedBy }) =>
          node && !node.complete && blockedBy.length
            ? notice(
              { kind: 'warning', title: `Waits for ${blockedBy.length} item(s)` },
              stack(
                { gap: 1 },
                ...blockedBy.map((entry) =>
                  flex(
                    { gap: 2, verticalAlign: 'center' },
                    link({ href: nodeHref(entry) }, `${refName(entry.key)} - ${entry.title}`),
                    statusTag(entry),
                  )
                ),
              ),
            )
            : null
        ),
        failure.view((message) => message && notice({ kind: 'error', dismissible: true }, message)),
        hasDirect.view((any) =>
          any
            ? list({
              label: 'Direct dependencies',
              items: directList,
              actions: (item) =>
                button(
                  {
                    size: 'sm',
                    variant: 'ghost',
                    label: `Remove the dependency on ${refName(item.id)}`,
                    events: { click: () => remove(item.id) },
                  },
                  'Remove',
                ),
            })
            : text({ size: 'sm', tone: 'secondary' }, 'This item depends on nothing.')
        ),
        tabs({
          label: 'Dependency trees',
          items: [
            {
              id: 'depends-on',
              label: 'Depends on',
              content: depTree({
                graph,
                root: itemKey,
                direction: 'dependsOn',
                label: 'Everything that this item depends on',
                empty: 'Nothing. This item can start now.',
                onRemove: remove,
              }),
            },
            {
              id: 'needed-by',
              label: 'Needed by',
              content: depTree({
                graph,
                root: itemKey,
                direction: 'neededBy',
                label: 'Everything that waits for this item',
                empty: 'Nothing waits for this item.',
              }),
            },
          ],
        }),
        dialog(
          { title: 'Add dependency', open: adding },
          adding.view((open) =>
            open ? addForm({ itemKey, onDone: () => adding.set(false) }) : null
          ),
        ),
      ),
    )
  },
})
