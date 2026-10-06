import { component } from 'imp'
import { div } from 'imp/html'
import { computed, initState } from 'imp/state'
import { button, link, text } from 'imp/std'
import { tree } from 'imp/std/data'
import { tag } from 'imp/std/display'
import { flex, stack } from 'imp/std/layout'
import { luCircle, luCircleCheck, luCircleDot, luGitMerge, luGitPullRequest } from 'imp/icons'
import {
  countItems,
  dependencyTree,
  itemIds,
  keyOfPath,
  nodeHref,
  nodeMap,
  parentIds,
  pathDepth,
  refName,
} from '../lib/graph.js'
import { STATUS_COLORS, STATUS_LABELS } from '../lib/api.js'

const PR_ICONS = { open: luGitPullRequest, merged: luGitMerge, closed: luGitPullRequest }

const TODO_ICONS = {
  'todo': luCircle,
  'in-progress': luCircleDot,
  'review': luCircleDot,
  'done': luCircleCheck,
}

const iconOf = (node) => (node.kind === 'pr' ? PR_ICONS[node.status] : TODO_ICONS[node.status])

const PR_TAGS = { open: 'blue', merged: 'violet', closed: 'gray' }

export const statusTag = (node) =>
  node.kind === 'pr'
    ? tag({ color: PR_TAGS[node.status], size: 'sm' }, node.status)
    : tag({ color: STATUS_COLORS[node.status], size: 'sm' }, STATUS_LABELS[node.status])

const selectedPanel = (map, path, { root, onRemove }) => {
  const key = keyOfPath(path)
  const node = map.get(key)
  if (!node) return text({ size: 'sm', tone: 'secondary' }, `${refName(key)} is missing.`)
  const direct = root && pathDepth(path) === 1
  return flex(
    { wrap: true, verticalAlign: 'center', gap: 2 },
    link({ href: nodeHref(node) }, `${refName(key)} - ${node.title}`),
    statusTag(node),
    !node.complete && node.blockedBy.length
      ? tag({ color: 'red', size: 'sm' }, `waits for ${node.blockedBy.length}`)
      : null,
    text(
      { inline: true, size: 'sm', tone: 'secondary' },
      `level ${pathDepth(path) + (root ? 0 : 1)}`,
    ),
    direct && onRemove
      ? button(
        { size: 'sm', variant: 'ghost', events: { click: () => onRemove(key) } },
        'Remove this link',
      )
      : null,
  )
}

export const depTree = component('todoer-dep-tree', {
  styles: `
    :host { display: block; }
    .scroll {
      overflow: auto;
      max-block-size: 34rem;
      border: var(--imp-border-width, 1px) solid var(--imp-color-border, #cbd5e1);
      border-radius: var(--imp-radius-item, 6px);
      padding: var(--imp-space-2, 8px);
      background: var(--imp-color-surface, #fff);
    }
    .inner { inline-size: max-content; min-inline-size: 100%; --imp-tree-indent: 1rem; }
  `,
  setup: (self, { graph, root = null, roots = null, direction, label, empty, onRemove }) => {
    const expanded = initState(self, 'todoer.tree.expanded', [])
    const selected = initState(self, 'todoer.tree.selected', null)
    const map = computed([graph], (nodes) => nodeMap(nodes))
    const build = (current, open) =>
      dependencyTree(current, {
        root,
        roots: roots ? roots(current) : [],
        direction,
        expanded: open,
        iconOf,
      })
    const items = computed([map, expanded], build)
    const full = computed([map], (current) => build(current, 'all'))
    const hasItems = computed([full], (all) => all.length > 0)
    const summary = computed([full], (all) => {
      const deepest = itemIds(all).reduce((max, id) => Math.max(max, pathDepth(id)), 0)
      return { count: countItems(all), levels: deepest + (root ? 0 : 1) }
    })
    const panel = computed([selected, map], (path, current) => ({ path, current }))
    return hasItems.view((any) =>
      any
        ? stack(
          { gap: 2 },
          flex(
            { horizontalAlign: 'justify', verticalAlign: 'center', wrap: true },
            summary.view(({ count, levels }) =>
              text(
                { size: 'sm', tone: 'secondary' },
                `${count} item(s) in ${levels} level(s). Open a row to see the next level.`,
              )
            ),
            flex(
              { gap: 2 },
              button(
                { size: 'sm', events: { click: () => expanded.set(parentIds(full.get())) } },
                'Expand all',
              ),
              button({ size: 'sm', events: { click: () => expanded.set([]) } }, 'Collapse all'),
            ),
          ),
          div(
            { class: 'scroll' },
            div({ class: 'inner' }, tree({ label, items, expanded, selected })),
          ),
          panel.view(({ path, current }) =>
            path && !path.endsWith('>...')
              ? selectedPanel(current, path, { root, onRemove })
              : text({ size: 'sm', tone: 'secondary' }, 'Select a row to see it, or to open it.')
          ),
        )
        : text({ size: 'sm', tone: 'secondary' }, empty)
    )
  },
})
