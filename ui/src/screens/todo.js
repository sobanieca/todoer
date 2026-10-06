import { component } from 'imp'
import { useHttpClient } from 'imp/http'
import { beforeLeave, navigate } from 'imp/router'
import { a, button as plainButton, div, figcaption, figure, img, span } from 'imp/html'
import { computed, initState, useState } from 'imp/state'
import { button, heading, icon, link, text } from 'imp/std'
import { card, descriptionList, markdown, tag } from 'imp/std/display'
import { notice, notify } from 'imp/std/feedback'
import { comboBox, fileUpload, select, textField } from 'imp/std/inputs'
import { dialog, flex, stack, tabs } from 'imp/std/layout'
import { luBot, luTrash2 } from 'imp/icons'
import { app } from '../app-map.js'
import {
  appendImage,
  assetUrl,
  formatKb,
  formatTime,
  imageFromClipboard,
  messageOf,
  previewMarkdown,
  STATUS_COLORS,
  STATUS_LABELS,
  uploadImage,
} from '../lib/api.js'
import { agentDialog } from '../parts/agent-dialog.js'
import { dependenciesCard } from '../parts/dependencies-card.js'
import { groupNames } from '../lib/graph.js'
import { groupHref } from './groups.js'
import { page } from '../parts/page.js'

const imageGrid = component('todoer-image-grid', {
  styles: `
    :host { display: block; }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(11rem, 1fr));
      gap: var(--imp-space-3, 12px);
    }
    figure {
      margin: 0;
      display: flex;
      flex-direction: column;
      gap: var(--imp-space-2, 8px);
      border: var(--imp-border-width, 1px) solid var(--imp-color-border, #cbd5e1);
      border-radius: var(--imp-radius-item, 6px);
      padding: var(--imp-space-2, 8px);
      background: var(--imp-color-surface-sunken, #f1f5f9);
      font-family: var(--imp-font-family, system-ui, sans-serif);
    }
    a { display: block; border-radius: var(--imp-radius-item, 6px); }
    a:focus-visible {
      outline: var(--imp-focus-ring-width, 2px) solid var(--imp-color-primary, #2563eb);
    }
    img {
      display: block;
      inline-size: 100%;
      block-size: 7rem;
      object-fit: contain;
      background: var(--imp-color-surface, #fff);
      border-radius: var(--imp-radius-item, 6px);
    }
    figcaption {
      display: flex;
      flex-direction: column;
      gap: var(--imp-space-1, 4px);
      font-size: var(--imp-font-size-sm, 0.875rem);
      color: var(--imp-color-secondary, #64748b);
    }
    .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
    .actions { display: flex; gap: var(--imp-space-2, 8px); }
    .action {
      border: none;
      background: none;
      color: var(--imp-color-primary, #2563eb);
      cursor: pointer;
      font: inherit;
      padding: 0;
      border-radius: var(--imp-radius-item, 6px);
    }
    .action.danger { color: var(--imp-color-danger, #dc2626); }
    .action:focus-visible {
      outline: var(--imp-focus-ring-width, 2px) solid var(--imp-color-primary, #2563eb);
      outline-offset: 2px;
    }
  `,
  setup: (_self, { todoId, images, onRemove, onInsert }) =>
    div(
      { class: 'grid' },
      images.map((image) =>
        figure(
          a(
            { href: assetUrl(todoId, image.name), target: '_blank', rel: 'noopener' },
            img({ src: assetUrl(todoId, image.name), alt: image.name }),
          ),
          figcaption(
            span({ class: 'name', title: image.name }, `${image.name} - ${formatKb(image.size)}`),
            span(
              { class: 'actions' },
              plainButton(
                {
                  class: 'action',
                  'aria-label': `Insert ${image.name} into the description`,
                  events: { click: () => onInsert(image) },
                },
                'Insert',
              ),
              plainButton(
                {
                  class: 'action danger',
                  'aria-label': `Remove ${image.name}`,
                  events: { click: () => onRemove(image) },
                },
                'Remove',
              ),
            ),
          ),
        )
      ),
    ),
})

const pasteZone = component('todoer-paste-zone', {
  styles: ':host { display: block; }',
  setup: (_self, { onImage }, child) =>
    div({
      events: {
        paste: (event) => {
          const file = imageFromClipboard(event)
          if (!file) return
          event.preventDefault()
          onImage(file)
        },
      },
    }, child),
})

const statusOptions = (statuses) =>
  statuses.map((value) => ({ value, label: STATUS_LABELS[value] ?? value }))

export const todoScreen = component('todoer-todo', {
  loading: () => text('Loading the todo...'),
  error: (_self, error) =>
    stack(
      { gap: 3 },
      notice({ kind: 'error', title: 'This todo cannot be shown' }, messageOf(error)),
      link({ href: app.todos.href() }, 'Back to the todos'),
    ),
  setup: async (self, { id }) => {
    const api = useHttpClient(self, 'todoer.api')
    const first = await api.get(`/todos/${id}`)
    if (self.abortSignal.aborted) return null
    const project = useState(self, 'todoer.project')
    const revision = useState(self, 'todoer.revision')
    const { maxImageBytes, maxImageKb, statuses } = project.get()
    const saved = initState(self, 'todoer.todo.saved', first)
    const title = initState(self, 'todoer.todo.title', first.title)
    const body = initState(self, 'todoer.todo.body', first.body)
    const status = initState(self, 'todoer.todo.status', first.status)
    const failure = initState(self, 'todoer.todo.failure', null)
    const saving = initState(self, 'todoer.todo.saving', false)
    const asking = initState(self, 'todoer.todo.asking', false)
    const removing = initState(self, 'todoer.todo.removing', false)
    const picked = initState(self, 'todoer.todo.picked', [])
    const graph = useState(self, 'todoer.graph')
    const groups = initState(self, 'todoer.todo.groups', first.groups)
    const groupOptions = computed([graph], (nodes) => groupNames(nodes))
    groups.watch(self, async (value) => {
      const current = saved.get().groups
      if (
        value.length === current.length && value.every((name, index) => name === current[index])
      ) {
        return
      }
      try {
        saved.set(await api.patch(`/todos/${id}`, { body: { groups: value } }))
        revision.set((count) => count + 1)
      } catch (error) {
        failure.set(messageOf(error))
        groups.set(saved.get().groups)
      }
    })
    const dirty = computed(
      [saved, title, body],
      (todo, nextTitle, nextBody) => todo.title !== nextTitle || todo.body !== nextBody,
    )
    const clean = computed([dirty], (value) => !value)
    beforeLeave(self, () => !dirty.get() || confirm('Discard the changes to this todo?'))
    const reload = async () => {
      let todo
      try {
        todo = await api.get(`/todos/${id}`)
      } catch {
        return
      }
      if (self.abortSignal.aborted) return
      const wasDirty = dirty.get()
      saved.set(todo)
      status.set(todo.status)
      groups.set(todo.groups)
      if (!wasDirty) {
        title.set(todo.title)
        body.set(todo.body)
      }
    }
    revision.watch(self, reload)
    status.watch(self, async (value) => {
      if (value === saved.get().status) return
      try {
        saved.set(await api.patch(`/todos/${id}`, { body: { status: value } }))
      } catch (error) {
        failure.set(messageOf(error))
        status.set(saved.get().status)
      }
    })
    const save = async () => {
      saving.set(true)
      try {
        const todo = await api.patch(`/todos/${id}`, {
          body: { title: title.get(), body: body.get() },
        })
        saved.set(todo)
        title.set(todo.title)
        body.set(todo.body)
        failure.set(null)
        notify({ kind: 'success', title: 'Saved' })
      } catch (error) {
        failure.set(messageOf(error))
      } finally {
        saving.set(false)
      }
    }
    const saveBody = async (next) => {
      body.set(next)
      const todo = await api.patch(`/todos/${id}`, { body: { body: next } })
      saved.set(todo)
    }
    const uploaded = new WeakSet()
    const upload = async (file) => {
      if (uploaded.has(file)) return
      uploaded.add(file)
      if (file.size > maxImageBytes) {
        notify({
          kind: 'error',
          title: `${file.name || 'The image'} is too large`,
          description: `It has ${formatKb(file.size)}. The limit is ${maxImageKb} kB ` +
            '(maxImageKb in todoer.json).',
        })
        return
      }
      try {
        const image = await uploadImage(api, id, file)
        await saveBody(appendImage(body.get(), image.markdown))
        notify({ kind: 'success', title: `Added ${image.name}` })
      } catch (error) {
        notify({ kind: 'error', title: 'The image was not added', description: messageOf(error) })
      }
    }
    const removeImage = async (image) => {
      if (!confirm(`Remove ${image.name}? The description keeps a broken link to it.`)) return
      try {
        await api.delete(`/todos/${id}/images/${encodeURIComponent(image.name)}`)
        await reload()
      } catch (error) {
        failure.set(messageOf(error))
      }
    }
    const insertImage = (image) => body.set(appendImage(body.get(), image.markdown))
    const removeTodo = async () => {
      try {
        await api.delete(`/todos/${id}`)
        removing.set(false)
        notify({ kind: 'success', title: `Removed todo ${id}` })
        saved.set({ ...saved.get(), title: title.get(), body: body.get() })
        navigate(app.todos.href())
      } catch (error) {
        failure.set(messageOf(error))
        removing.set(false)
      }
    }
    return page(
      flex(
        { horizontalAlign: 'justify', verticalAlign: 'center', wrap: true },
        stack(
          { gap: 1 },
          heading(saved.text((todo) => todo.title)),
          saved.view((todo) =>
            flex(
              { gap: 2, verticalAlign: 'center', wrap: true },
              tag({ color: STATUS_COLORS[todo.status], size: 'sm' }, STATUS_LABELS[todo.status]),
              text({ inline: true, size: 'sm', tone: 'secondary' }, todo.id),
              todo.pr
                ? link(
                  { href: app.prs.$prId.href({ prId: todo.pr }), size: 'sm' },
                  `Pull request ${todo.pr}`,
                )
                : null,
              ...todo.groups.map((name) =>
                link({ href: groupHref(name), size: 'sm' }, `# ${name}`)
              ),
            )
          ),
        ),
        flex(
          { gap: 2 },
          button(
            { events: { click: () => asking.set(true) } },
            icon(luBot, { size: 'sm' }),
            'Ask agent',
          ),
          button(
            {
              variant: 'danger',
              label: 'Remove the todo',
              square: true,
              events: { click: () => removing.set(true) },
            },
            icon(luTrash2, { size: 'sm' }),
          ),
        ),
      ),
      failure.view((message) => message && notice({ kind: 'error', dismissible: true }, message)),
      card(
        stack(
          { gap: 4 },
          flex(
            { wrap: true, verticalAlign: 'top' },
            div(
              { style: { flex: '1 1 24rem' } },
              textField({ label: 'Title', value: title, maxLength: 200 }),
            ),
            select({ label: 'Status', value: status, options: statusOptions(statuses) }),
          ),
          groupOptions.view((options) =>
            comboBox({
              label: 'Groups',
              value: groups,
              options,
              multiple: true,
              custom: true,
              placeholder: 'Type a group name, then press Enter',
              help: 'A todo can be in many groups. A new name makes a new group.',
            })
          ),
          tabs({
            label: 'Description',
            items: [
              {
                id: 'write',
                label: 'Write',
                content: pasteZone(
                  { onImage: upload },
                  textField({
                    label: 'Description',
                    value: body,
                    rows: 14,
                    help: `Markdown. Paste an image to attach it (${maxImageKb} kB or less).`,
                  }),
                ),
              },
              {
                id: 'preview',
                label: 'Preview',
                content: markdown({ text: computed([body], previewMarkdown) }),
              },
            ],
          }),
          flex(
            { horizontalAlign: 'end', verticalAlign: 'center' },
            dirty.view((changed) =>
              changed ? text({ size: 'sm', tone: 'secondary' }, 'Unsaved changes') : null
            ),
            button({
              variant: 'primary',
              disabled: clean,
              loading: saving,
              events: { click: save },
            }, 'Save'),
          ),
        ),
      ),
      card(
        { header: heading({ level: 2 }, 'Images') },
        stack(
          { gap: 4 },
          fileUpload({
            label: 'Add images',
            value: picked,
            multiple: true,
            accept: 'image/png,image/jpeg,image/gif,image/webp',
            maxSize: maxImageBytes,
            help: `maxImageKb in todoer.json sets the limit of ${maxImageKb} kB.`,
            onFiles: async (files) => {
              const fresh = files.filter((file) => !uploaded.has(file))
              for (const file of fresh) await upload(file)
              if (fresh.length) picked.set([])
            },
          }),
          saved.view((todo) =>
            todo.images.length
              ? imageGrid({
                todoId: todo.id,
                images: todo.images,
                onRemove: removeImage,
                onInsert: insertImage,
              })
              : text({ size: 'sm', tone: 'secondary' }, 'No images.')
          ),
        ),
      ),
      dependenciesCard({ itemKey: `todo:${id}` }),
      card(
        { header: heading({ level: 2 }, 'Details') },
        saved.view((todo) =>
          descriptionList({
            items: [
              { term: 'Id', details: todo.id },
              { term: 'Created', details: formatTime(todo.created) },
              { term: 'Updated', details: formatTime(todo.updated) },
              { term: 'Branch', details: todo.branch ?? '-' },
              { term: 'Worktree', details: todo.worktree ?? '-' },
              { term: 'File', details: `.todoer/todos/${todo.file}` },
            ],
          })
        ),
      ),
      agentDialog({ open: asking, request: () => ({ todos: [id] }) }),
      dialog(
        { title: 'Remove the todo?', open: removing },
        stack(
          { gap: 4 },
          text(`This removes the todo ${id} and its images. You cannot undo it.`),
          flex(
            { horizontalAlign: 'end' },
            button({ events: { click: () => removing.set(false) } }, 'Cancel'),
            button({ variant: 'danger', events: { click: removeTodo } }, 'Remove'),
          ),
        ),
      ),
    )
  },
})
