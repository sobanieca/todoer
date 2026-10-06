import { expect, test } from 'imp/test'
import { appendImage, previewMarkdown } from '../src/lib/api.js'
import {
  canDependOn,
  countItems,
  dependencyOptions,
  dependencyTree,
  groupRoots,
  groupSlug,
  keyOfPath,
  nodeMap,
  parentIds,
  pathDepth,
} from '../src/lib/graph.js'
import {
  commentsOfFile,
  commentsOfFolder,
  commentsOfLine,
  fileTree,
  folderCounts,
  folderIds,
  treeItems,
} from '../src/lib/review.js'

const files = [
  { path: 'src/app.js', oldPath: 'src/app.js', status: 'modified', additions: 2, deletions: 1 },
  {
    path: 'src/legacy/d.js',
    oldPath: 'src/legacy/b.js',
    status: 'renamed',
    additions: 0,
    deletions: 0,
  },
  { path: 'README.md', oldPath: null, status: 'added', additions: 3, deletions: 0 },
]

const comments = [
  { id: 'c-1', kind: 'line', path: 'src/app.js', line: 4, side: 'new', target: 'src/app.js:4' },
  { id: 'c-2', kind: 'line', path: 'src/app.js', line: 4, side: 'old', target: 'src/app.js:old:4' },
  { id: 'c-3', kind: 'file', path: 'src/legacy/b.js', target: 'src/legacy/b.js' },
  { id: 'c-4', kind: 'folder', path: 'src/legacy/', target: 'src/legacy/' },
]

test('builds a tree with the folders first', () => {
  const tree = fileTree(files)
  expect(tree.map((node) => node.name)).toEqual(['src', 'README.md'])
  expect(tree[0].children.map((node) => node.name)).toEqual(['legacy', 'app.js'])
  expect(tree[0].children[0].path).toEqual('src/legacy/')
})

test('labels the tree items with the changes and the comment counts', () => {
  const items = treeItems(files, comments)
  expect(items[0].id).toEqual('dir:src/')
  expect(items[0].children[0].label).toEqual('legacy/  (1)')
  expect(items[0].children[0].children[0].label).toEqual('d.js  +0 -0 renamed  (1)')
  expect(items[0].children[1].label).toEqual('app.js  +2 -1  (2)')
  expect(folderIds(items)).toEqual(['dir:src/', 'dir:src/legacy/'])
})

test('finds the comments of a file by its old or new path', () => {
  expect(commentsOfFile(comments, files[1]).map((comment) => comment.id)).toEqual(['c-3'])
  expect(commentsOfFolder(comments, 'src/legacy').map((comment) => comment.id)).toEqual(['c-4'])
  expect(folderCounts(comments)).toEqual({ 'src/legacy/': 1 })
})

test('puts a line comment on the side that it names', () => {
  const added = { kind: 'add', old: null, new: 4 }
  const removed = { kind: 'del', old: 4, new: null }
  const context = { kind: 'context', old: 4, new: 5 }
  expect(commentsOfLine(comments, added).map((comment) => comment.id)).toEqual(['c-1'])
  expect(commentsOfLine(comments, removed).map((comment) => comment.id)).toEqual(['c-2'])
  expect(commentsOfLine(comments, context).map((comment) => comment.id)).toEqual(['c-2'])
})

test('shows the images of a todo in the preview', () => {
  expect(previewMarkdown('see ![a](../assets/ab12/1-shot.png)')).toEqual(
    'see ![a](/files/assets/ab12/1-shot.png)',
  )
  expect(appendImage('text  \n', '![x](y)')).toEqual('text\n\n![x](y)')
  expect(appendImage('', '![x](y)')).toEqual('![x](y)')
})

const graphNode = (key, dependsOn = [], extra = {}) => ({
  key,
  kind: key.split(':')[0],
  id: key.split(':')[1],
  title: key,
  status: 'todo',
  complete: false,
  groups: [],
  dependsOn,
  todos: [],
  neededBy: [],
  blockedBy: [],
  ...extra,
})

const withNeededBy = (nodes) => {
  for (const node of nodes) {
    for (const key of node.dependsOn) {
      nodes.find((entry) => entry.key === key)?.neededBy.push(node.key)
    }
  }
  return nodeMap(nodes)
}

test('builds a lazy dependency tree that opens one level at a time', () => {
  const chain = Array.from(
    { length: 200 },
    (_, index) =>
      graphNode(
        `todo:t${String(index).padStart(3, '0')}`,
        index ? [`todo:t${String(index - 1).padStart(3, '0')}`] : [],
      ),
  )
  const map = withNeededBy(chain)
  const closed = dependencyTree(map, { root: 'todo:t199', direction: 'dependsOn', expanded: [] })
  expect(closed.length).toEqual(1)
  expect(closed[0].children).toEqual([{ id: 'todo:t199>todo:t198>...', label: 'More' }])
  const all = dependencyTree(map, { root: 'todo:t199', direction: 'dependsOn', expanded: 'all' })
  expect(countItems(all)).toEqual(199)
  expect(parentIds(all).length).toEqual(198)
  expect(keyOfPath(parentIds(all).at(-1))).toEqual('todo:t001')
  expect(pathDepth(parentIds(all).at(-1))).toEqual(198)
  const up = dependencyTree(map, { root: 'todo:t000', direction: 'neededBy', expanded: 'all' })
  expect(countItems(up)).toEqual(199)
})

test('shows a shared item once and marks the next path', () => {
  const map = withNeededBy([
    graphNode('todo:deep'),
    graphNode('todo:base', ['todo:deep']),
    graphNode('todo:left', ['todo:base']),
    graphNode('todo:right', ['todo:base']),
    graphNode('todo:top', ['todo:left', 'todo:right']),
  ])
  const items = dependencyTree(map, { root: 'todo:top', direction: 'dependsOn', expanded: 'all' })
  expect(items[1].children[0].label).toMatch('see above')
  expect(items[1].children[0].children).toEqual(undefined)
  expect(countItems(items)).toEqual(5)
})

test('offers only dependencies that make no cycle', () => {
  const map = withNeededBy([
    graphNode('todo:a'),
    graphNode('todo:b', ['todo:a']),
    graphNode('todo:c', ['todo:b']),
    graphNode('pr:p1', [], { todos: ['todo:a'] }),
  ])
  expect(canDependOn(map, 'todo:a', 'todo:c')).toEqual(false)
  expect(canDependOn(map, 'todo:c', 'todo:a')).toEqual(true)
  expect(canDependOn(map, 'pr:p1', 'todo:a')).toEqual(false)
  expect(dependencyOptions(map, 'todo:a').map((option) => option.value)).toEqual([])
  expect(dependencyOptions(map, 'todo:b').map((option) => option.value)).toEqual(['pr:p1'])
  expect(groupRoots(map, ['todo:a', 'todo:b', 'todo:c'])).toEqual(['todo:c'])
  expect(groupSlug('Bump @scope/pkg to 0.3.5')).toEqual('bump-scope-pkg-to-0-3-5')
})
