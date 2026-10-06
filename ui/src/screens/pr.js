import { component } from 'imp'
import { useHttpClient } from 'imp/http'
import { div } from 'imp/html'
import { computed, initState, useState } from 'imp/state'
import { button, heading, link, menuButton, text } from 'imp/std'
import { tree } from 'imp/std/data'
import { card, codeBlock, emptyState, markdown, tag } from 'imp/std/display'
import { notice, notify } from 'imp/std/feedback'
import { checkbox, select, textField } from 'imp/std/inputs'
import { dialog, flex, splitPane, stack } from 'imp/std/layout'
import { app } from '../app-map.js'
import { messageOf, PR_COLORS, VERDICT_COLORS, VERDICT_LABELS } from '../lib/api.js'
import { commentsOfFolder, folderIds, folderTarget, treeItems } from '../lib/review.js'
import { agentDialog } from '../parts/agent-dialog.js'
import { dependenciesCard } from '../parts/dependencies-card.js'
import { nodeMap, refName } from '../lib/graph.js'
import { commentList, diffFile } from '../parts/diff-file.js'
import { page } from '../parts/page.js'

const composer = component('todoer-comment-composer', {
  setup: (self, { draft, onSave, onCancel }) => {
    const value = initState(self, 'todoer.composer.text', draft.text ?? '')
    const busy = initState(self, 'todoer.composer.busy', false)
    const failure = initState(self, 'todoer.composer.failure', null)
    const empty = computed([value], (current) => !current.trim())
    const save = async () => {
      busy.set(true)
      try {
        await onSave(value.get())
      } catch (error) {
        failure.set(messageOf(error))
      } finally {
        if (!self.abortSignal.aborted) busy.set(false)
      }
    }
    return stack(
      { gap: 3 },
      text({ size: 'sm', tone: 'secondary' }, `Target: ${draft.target}`),
      draft.quote !== null && draft.quote !== undefined
        ? codeBlock({ code: draft.quote || ' ', label: draft.target })
        : null,
      textField({
        label: 'Comment',
        value,
        rows: 6,
        help: draft.target.endsWith('/')
          ? 'A comment on the whole folder, for example "move it under lib/".'
          : draft.quote === null
          ? 'A comment on the whole file, for example "rename it" or "move it".'
          : 'Markdown.',
      }),
      failure.view((message) => message && notice({ kind: 'error' }, message)),
      flex(
        { horizontalAlign: 'end' },
        button({ events: { click: onCancel } }, 'Cancel'),
        button(
          { variant: 'primary', disabled: empty, loading: busy, events: { click: save } },
          'Save',
        ),
      ),
    )
  },
})

const changesForm = component('todoer-changes-form', {
  setup: (self, { summary, onSubmit, onCancel }) => {
    const value = initState(self, 'todoer.changes.summary', summary ?? '')
    const busy = initState(self, 'todoer.changes.busy', false)
    const failure = initState(self, 'todoer.changes.failure', null)
    const submit = async () => {
      busy.set(true)
      try {
        await onSubmit(value.get())
      } catch (error) {
        failure.set(messageOf(error))
        busy.set(false)
      }
    }
    return stack(
      { gap: 3 },
      text(
        { size: 'sm', tone: 'secondary' },
        'The agent reads the summary and every comment with "todoer pr show", fixes the code ' +
          'in the same worktree, and submits the next revision.',
      ),
      textField({ label: 'Summary (optional)', value, rows: 4 }),
      failure.view((message) => message && notice({ kind: 'error' }, message)),
      flex(
        { horizontalAlign: 'end' },
        button({ events: { click: onCancel } }, 'Cancel'),
        button({ variant: 'primary', loading: busy, events: { click: submit } }, 'Request changes'),
      ),
    )
  },
})

const mergeForm = component('todoer-merge-form', {
  setup: (self, { mainBranch, blockers, onMerge, onCancel }) => {
    const cleanup = initState(self, 'todoer.merge.cleanup', true)
    const force = initState(self, 'todoer.merge.force', false)
    const stopped = computed([force], (value) => blockers.length > 0 && !value)
    const busy = initState(self, 'todoer.merge.busy', false)
    const failure = initState(self, 'todoer.merge.failure', null)
    const merge = async () => {
      busy.set(true)
      failure.set(null)
      try {
        await onMerge({ keep: !cleanup.get(), force: force.get() })
      } catch (error) {
        failure.set(messageOf(error))
        busy.set(false)
      }
    }
    return stack(
      { gap: 3 },
      text(
        `todoer applies the patch on ${mainBranch} with "git am --3way" in the main checkout. ` +
          'The commits keep their author and their message.',
      ),
      blockers.length
        ? notice(
          { kind: 'warning', title: `This pull request waits for ${blockers.length} item(s)` },
          blockers.join(', '),
        )
        : null,
      blockers.length ? checkbox({ label: 'Merge anyway', value: force }) : null,
      checkbox({ label: 'Remove the worktree and the branch after the merge', value: cleanup }),
      failure.view((message) =>
        message &&
        notice(
          { kind: 'error', title: 'The merge failed' },
          div({ style: { whiteSpace: 'pre-wrap' } }, message),
        )
      ),
      flex(
        { horizontalAlign: 'end' },
        button({ events: { click: onCancel } }, 'Cancel'),
        button(
          { variant: 'primary', loading: busy, disabled: stopped, events: { click: merge } },
          'Approve and merge',
        ),
      ),
    )
  },
})

const prHeader = (details, { choice }) => {
  const { pr, review } = details
  return flex(
    { horizontalAlign: 'justify', verticalAlign: 'top', wrap: true },
    stack(
      { gap: 2 },
      heading(pr.title),
      flex(
        { gap: 2, wrap: true, verticalAlign: 'center' },
        tag({ color: PR_COLORS[pr.status], size: 'sm' }, pr.status),
        tag({ color: VERDICT_COLORS[review.verdict], size: 'sm' }, VERDICT_LABELS[review.verdict]),
        text({ inline: true, size: 'sm', tone: 'secondary' }, `${pr.id} - ${pr.branch ?? '-'}`),
        ...pr.todos.map((todoId) =>
          link({ href: app.todos.$todoId.href({ todoId }), size: 'sm' }, `todo ${todoId}`)
        ),
      ),
    ),
    select({
      label: 'Revision',
      value: choice,
      options: details.revisions.map((number) => ({
        value: String(number),
        label: number === pr.revision ? `v${number} (latest)` : `v${number}`,
      })),
    }),
  )
}

export const prScreen = component('todoer-pr', {
  loading: () => text('Loading the pull request...'),
  error: (_self, error) =>
    stack(
      { gap: 3 },
      notice({ kind: 'error', title: 'This pull request cannot be shown' }, messageOf(error)),
      link({ href: app.prs.href() }, 'Back to the pull requests'),
    ),
  setup: async (self, { id }) => {
    const api = useHttpClient(self, 'todoer.api')
    const first = await api.get(`/prs/${id}`)
    if (self.abortSignal.aborted) return null
    const project = useState(self, 'todoer.project')
    const graph = useState(self, 'todoer.graph')
    const changes = useState(self, 'todoer.revision')
    const details = initState(self, 'todoer.pr.details', first)
    const chosen = initState(self, 'todoer.pr.chosen', first.revision)
    const choice = initState(self, 'todoer.pr.choice', String(first.revision))
    const folder = initState(self, 'todoer.pr.folder', null)
    const draft = initState(self, 'todoer.pr.draft', null)
    const composing = initState(self, 'todoer.pr.composing', false)
    const requesting = initState(self, 'todoer.pr.requesting', false)
    const merging = initState(self, 'todoer.pr.merging', false)
    const asking = initState(self, 'todoer.pr.asking', false)
    const failure = initState(self, 'todoer.pr.failure', null)
    const files = new Map()
    const load = async () => {
      const latest = details.get().pr.revision
      const follow = chosen.get() === latest
      let next
      try {
        next = await api.get(`/prs/${id}`, {
          query: { revision: follow ? undefined : chosen.get() },
        })
      } catch (error) {
        failure.set(messageOf(error))
        return
      }
      if (self.abortSignal.aborted) return
      if (follow && next.revision !== chosen.get()) chosen.set(next.revision)
      choice.set(String(next.revision))
      details.set(next)
    }
    changes.watch(self, load)
    const editableOf = (value) => value.pr.status === 'open' && value.revision === value.pr.revision
    const openComposer = (next) => {
      draft.set(next)
      composing.set(true)
    }
    const editComment = (comment) =>
      openComposer({
        target: comment.target,
        quote: comment.kind === 'line' ? comment.quote : null,
        text: comment.text,
        commentId: comment.id,
      })
    const deleteComment = async (comment) => {
      if (!confirm(`Delete the comment on ${comment.target}?`)) return
      try {
        await api.delete(`/prs/${id}/comments/${comment.id}`)
        await load()
      } catch (error) {
        failure.set(messageOf(error))
      }
    }
    const saveComment = async (value) => {
      const current = draft.get()
      if (current.commentId) {
        await api.patch(`/prs/${id}/comments/${current.commentId}`, { body: { text: value } })
      } else {
        await api.post(`/prs/${id}/comments`, { body: { target: current.target, text: value } })
      }
      composing.set(false)
      draft.set(null)
      await load()
    }
    const handlers = {
      onComment: (next) => openComposer({ ...next, text: '' }),
      onEdit: editComment,
      onDelete: deleteComment,
      register: (path, host) => files.set(path, host),
    }
    const selectNode = (event) => {
      const nodeId = event.detail.id
      if (nodeId.startsWith('dir:')) folder.set(nodeId.slice(4))
      else files.get(nodeId.slice(5))?.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
    choice.watch(self, (value) => {
      if (Number(value) === chosen.get()) return
      chosen.set(Number(value))
      load()
    })
    const requestChanges = async (summary) => {
      await api.post(`/prs/${id}/review`, { body: { verdict: 'changes-requested', summary } })
      requesting.set(false)
      await load()
      asking.set(true)
    }
    const merge = async ({ keep, force }) => {
      await api.post(`/prs/${id}/review`, { body: { verdict: 'approved' } })
      const result = await api.post(`/prs/${id}/merge`, { body: { keep, force } })
      merging.set(false)
      notify({
        kind: 'success',
        title: `Merged into ${project.get().mainBranch}`,
        description: result.warnings.join(' ') || `Commit ${result.pr.merged.slice(0, 10)}`,
        duration: result.warnings.length ? 0 : undefined,
      })
      await load()
    }
    const closePr = async () => {
      if (!confirm('Close this pull request without a merge? Its todos are open again.')) return
      try {
        await api.post(`/prs/${id}/close`, { body: {} })
        await load()
      } catch (error) {
        failure.set(messageOf(error))
      }
    }
    const folderPanel = computed([details, folder], (value, path) => ({
      path,
      editable: editableOf(value),
      comments: path ? commentsOfFolder(value.review.comments, path) : [],
      all: value.review.comments.filter((comment) => comment.kind === 'folder'),
    }))
    const items = computed([details], (value) => treeItems(value.files, value.review.comments))
    return page(
      details.view((value) => prHeader(value, { choice })),
      failure.view((message) => message && notice({ kind: 'error', dismissible: true }, message)),
      details.view((value) => {
        const { pr } = value
        if (pr.status === 'merged') {
          return notice(
            { kind: 'success', title: 'Merged' },
            `The patch is on ${project.get().mainBranch} as ${pr.merged?.slice(0, 10)}.`,
          )
        }
        if (pr.status === 'closed') return notice({ kind: 'info', title: 'Closed without a merge' })
        if (value.revision !== pr.revision) {
          return notice(
            { kind: 'info', title: `You see revision ${value.revision} of ${pr.revision}` },
            'Comments go on the latest revision.',
          )
        }
        return null
      }),
      details.view((value) =>
        value.pr.description
          ? card(
            { header: heading({ level: 2 }, 'Description') },
            markdown({ text: value.pr.description }),
          )
          : null
      ),
      details.view((value) =>
        editableOf(value)
          ? flex(
            { horizontalAlign: 'justify', verticalAlign: 'center', wrap: true },
            text(
              { size: 'sm', tone: 'secondary' },
              `${value.review.comments.length} comment(s) on this revision. ` +
                'Click + beside a line, "Comment on file" on a file, or select a folder in the tree.',
            ),
            flex(
              { gap: 2 },
              button({ events: { click: () => requesting.set(true) } }, 'Request changes'),
              button(
                { variant: 'primary', events: { click: () => merging.set(true) } },
                'Approve and merge',
              ),
              menuButton({
                label: 'More actions',
                items: [
                  {
                    id: 'prompt',
                    label: 'Ask agent to fix the review',
                    onClick: () => asking.set(true),
                  },
                  { id: 'close', label: 'Close without a merge', onClick: closePr },
                ],
              }),
            ),
          )
          : null
      ),
      splitPane(
        { size: 24, min: 12, max: 45, label: 'Resize the file tree' },
        stack(
          { gap: 4 },
          card(
            { header: text({ weight: 'bold' }, 'Files and folders'), padding: 'sm' },
            items.view((list) =>
              list.length
                ? tree({
                  label: 'Changed files and folders',
                  items: list,
                  expanded: folderIds(list),
                  events: { select: selectNode },
                })
                : text({ size: 'sm', tone: 'secondary' }, 'No changed file.')
            ),
          ),
          card(
            { header: text({ weight: 'bold' }, 'Folder comments'), padding: 'sm' },
            folderPanel.view(({ path, editable, comments, all }) =>
              stack(
                { gap: 3 },
                path
                  ? stack(
                    { gap: 2 },
                    text({ size: 'sm' }, `Folder ${path}`),
                    editable
                      ? button(
                        {
                          size: 'sm',
                          events: {
                            click: () =>
                              openComposer({ target: folderTarget(path), quote: null, text: '' }),
                          },
                        },
                        'Comment on folder',
                      )
                      : null,
                  )
                  : text(
                    { size: 'sm', tone: 'secondary' },
                    'Select a folder in the tree to comment on it.',
                  ),
                (path ? comments : all).length
                  ? commentList({
                    comments: path ? comments : all,
                    editable,
                    onEdit: editComment,
                    onDelete: deleteComment,
                  })
                  : null,
              )
            ),
          ),
        ),
        div(details.view((value) =>
          value.files.length
            ? stack(
              { gap: 4 },
              ...value.files.map((file) =>
                diffFile({
                  file,
                  comments: value.review.comments,
                  canComment: editableOf(value),
                  ...handlers,
                })
              ),
            )
            : emptyState({ title: 'No change', description: 'This revision changes no file.' })
        )),
      ),
      dependenciesCard({ itemKey: `pr:${id}` }),
      details.view((value) =>
        value.previous.length
          ? card(
            { header: heading({ level: 2 }, 'Comments on earlier revisions') },
            stack(
              { gap: 4 },
              ...value.previous.map((old) =>
                stack(
                  { gap: 2 },
                  text(
                    { weight: 'bold' },
                    `Revision ${old.revision} - ${VERDICT_LABELS[old.verdict]}`,
                  ),
                  old.summary ? markdown({ text: old.summary }) : null,
                  commentList({ comments: old.comments, editable: false }),
                )
              ),
            ),
          )
          : null
      ),
      dialog(
        { title: 'Comment', open: composing },
        composing.view((open) =>
          open && draft.get()
            ? composer({
              draft: draft.get(),
              onSave: saveComment,
              onCancel: () => composing.set(false),
            })
            : null
        ),
      ),
      dialog(
        { title: 'Request changes', open: requesting },
        requesting.view((open) =>
          open
            ? changesForm({
              summary: details.get().review.summary,
              onSubmit: requestChanges,
              onCancel: () => requesting.set(false),
            })
            : null
        ),
      ),
      dialog(
        { title: 'Approve and merge', open: merging },
        merging.view((open) =>
          open
            ? mergeForm({
              mainBranch: project.get().mainBranch,
              blockers: (nodeMap(graph.get()).get(`pr:${id}`)?.blockedBy ?? []).map(refName),
              onMerge: merge,
              onCancel: () => merging.set(false),
            })
            : null
        ),
      ),
      agentDialog({ open: asking, request: () => ({ pr: id }) }),
    )
  },
})
