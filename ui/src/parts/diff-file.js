import { component } from 'imp'
import { button, code, div, header, p, section, span, strong } from 'imp/html'
import { markdown } from 'imp/std/display'
import { commentsOfFile, commentsOfLine, fileTarget, lineTarget } from '../lib/review.js'

const COMMENT_STYLES = `
  .comment {
    border: var(--imp-border-width, 1px) solid var(--imp-color-border, #cbd5e1);
    border-inline-start: 3px solid var(--imp-color-warning, #d97706);
    border-radius: var(--imp-radius-item, 6px);
    background: var(--imp-color-surface, #fff);
    padding: var(--imp-space-2, 8px) var(--imp-space-3, 12px);
    font-family: var(--imp-font-family, system-ui, sans-serif);
    color: var(--imp-color-text, #0f172a);
  }
  .comment-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--imp-space-2, 8px);
    font-size: var(--imp-font-size-sm, 0.875rem);
    color: var(--imp-color-secondary, #64748b);
  }
  .comment-target { font-family: var(--imp-font-family-mono, monospace); overflow-wrap: anywhere; }
  .comment-actions { display: flex; gap: var(--imp-space-2, 8px); }
  .link-button {
    border: none;
    background: none;
    padding: 0;
    font: inherit;
    font-size: var(--imp-font-size-sm, 0.875rem);
    color: var(--imp-color-primary, #2563eb);
    cursor: pointer;
    border-radius: var(--imp-radius-item, 6px);
  }
  .link-button.danger { color: var(--imp-color-danger, #dc2626); }
  .link-button:focus-visible {
    outline: var(--imp-focus-ring-width, 2px) solid var(--imp-color-primary, #2563eb);
    outline-offset: 2px;
  }
`

const commentCard = (comment, { editable, onEdit, onDelete }) =>
  div(
    { class: 'comment' },
    div(
      { class: 'comment-head' },
      span({ class: 'comment-target' }, comment.target),
      editable
        ? span(
          { class: 'comment-actions' },
          button(
            {
              class: 'link-button',
              'aria-label': `Edit the comment on ${comment.target}`,
              events: { click: () => onEdit(comment) },
            },
            'Edit',
          ),
          button(
            {
              class: 'link-button danger',
              'aria-label': `Delete the comment on ${comment.target}`,
              events: { click: () => onDelete(comment) },
            },
            'Delete',
          ),
        )
        : null,
    ),
    markdown({ text: comment.text }),
  )

const STATUS_TEXT = { added: 'added', deleted: 'deleted', renamed: 'renamed', modified: 'modified' }

const SIGNS = { add: '+', del: '-', context: ' ' }

export const diffFile = component('todoer-diff-file', {
  styles: `
    :host { display: block; }
    .file {
      border: var(--imp-border-width, 1px) solid var(--imp-color-border, #cbd5e1);
      border-radius: var(--imp-radius-container, 10px);
      background: var(--imp-color-surface, #fff);
      overflow: hidden;
    }
    .head {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: var(--imp-space-2, 8px);
      padding: var(--imp-space-2, 8px) var(--imp-space-3, 12px);
      background: var(--imp-color-surface-sunken, #f1f5f9);
      border-block-end: var(--imp-border-width, 1px) solid var(--imp-color-border, #cbd5e1);
      font-family: var(--imp-font-family, system-ui, sans-serif);
      font-size: var(--imp-font-size-sm, 0.875rem);
      color: var(--imp-color-text, #0f172a);
      position: sticky;
      inset-block-start: 0;
      z-index: 1;
    }
    .path {
      font-family: var(--imp-font-family-mono, monospace);
      font-weight: var(--imp-font-weight-bold, 600);
      overflow-wrap: anywhere;
      flex: 1 1 12rem;
    }
    .status { color: var(--imp-color-secondary, #64748b); }
    .added-count { color: var(--imp-color-success, #16a34a); }
    .deleted-count { color: var(--imp-color-danger, #dc2626); }
    .file-comments, .inline {
      display: flex;
      flex-direction: column;
      gap: var(--imp-space-2, 8px);
      padding: var(--imp-space-2, 8px) var(--imp-space-3, 12px);
      background: var(--imp-color-surface-sunken, #f1f5f9);
    }
    .file-comments { border-block-end: var(--imp-border-width, 1px) solid var(--imp-color-border, #cbd5e1); }
    .lines {
      font-family: var(--imp-font-family-mono, monospace);
      font-size: var(--imp-font-size-sm, 0.875rem);
      line-height: 1.5;
      overflow-x: auto;
    }
    .hunk {
      padding: var(--imp-space-1, 4px) var(--imp-space-3, 12px);
      color: var(--imp-color-secondary, #64748b);
      background: color-mix(in oklab, var(--imp-color-info, #2563eb) 8%, var(--imp-color-surface, #fff));
      white-space: pre;
    }
    .row {
      display: grid;
      grid-template-columns: 3.5rem 3.5rem 1.75rem 1.25rem max-content;
      min-inline-size: 100%;
      inline-size: max-content;
      color: var(--imp-color-text, #0f172a);
    }
    .row.add { background: color-mix(in oklab, var(--imp-color-success, #16a34a) 13%, var(--imp-color-surface, #fff)); }
    .row.del { background: color-mix(in oklab, var(--imp-color-danger, #dc2626) 13%, var(--imp-color-surface, #fff)); }
    .num {
      text-align: end;
      padding-inline-end: var(--imp-space-2, 8px);
      color: var(--imp-color-secondary, #64748b);
      user-select: none;
    }
    .add-comment {
      opacity: 0;
      border: none;
      border-radius: var(--imp-radius-item, 6px);
      background: var(--imp-color-primary, #2563eb);
      color: var(--imp-color-on-primary, #fff);
      font: inherit;
      line-height: 1.1;
      margin-block: 1px;
      cursor: pointer;
      padding: 0;
    }
    .row:hover .add-comment, .add-comment:focus-visible { opacity: 1; }
    .add-comment:focus-visible {
      outline: var(--imp-focus-ring-width, 2px) solid var(--imp-color-primary, #2563eb);
      outline-offset: 1px;
    }
    .sign { user-select: none; color: var(--imp-color-secondary, #64748b); }
    .text { white-space: pre; padding-inline-end: var(--imp-space-3, 12px); }
    .note {
      margin: 0;
      padding: var(--imp-space-3, 12px);
      color: var(--imp-color-secondary, #64748b);
      font-family: var(--imp-font-family, system-ui, sans-serif);
      font-size: var(--imp-font-size-sm, 0.875rem);
    }
    .small-button {
      border: var(--imp-border-width, 1px) solid var(--imp-color-border, #cbd5e1);
      border-radius: var(--imp-radius-item, 6px);
      background: var(--imp-color-surface, #fff);
      color: var(--imp-color-text, #0f172a);
      font: inherit;
      padding: 2px var(--imp-space-2, 8px);
      cursor: pointer;
    }
    .small-button:hover { background: var(--imp-color-surface-hover, #e2e8f0); }
    .small-button:focus-visible {
      outline: var(--imp-focus-ring-width, 2px) solid var(--imp-color-primary, #2563eb);
      outline-offset: 2px;
    }
    @media (forced-colors: active) {
      .row.add, .row.del { border-inline-start: 3px solid CanvasText; }
      .add-comment { opacity: 1; }
    }
    ${COMMENT_STYLES}
  `,
  setup: (self, { file, comments, canComment, onComment, onEdit, onDelete, register }) => {
    register?.(file.path, self.host)
    const own = commentsOfFile(comments, file)
    const fileComments = own.filter((comment) => comment.kind === 'file')
    const handlers = { editable: canComment, onEdit, onDelete }
    const lineRow = (line) => {
      const side = line.kind === 'del' ? 'old' : 'new'
      const number = side === 'old' ? line.old : line.new
      const path = side === 'old' ? file.oldPath ?? file.path : file.newPath ?? file.path
      const target = lineTarget(path, number, side)
      const inline = commentsOfLine(own, line)
      return [
        div(
          { class: ['row', line.kind] },
          span({ class: 'num' }, line.old ?? ''),
          span({ class: 'num' }, line.new ?? ''),
          canComment
            ? button(
              {
                class: 'add-comment',
                'aria-label': side === 'old'
                  ? `Comment on removed line ${number}`
                  : `Comment on line ${number}`,
                title: target,
                events: { click: () => onComment({ target, quote: line.text }) },
              },
              '+',
            )
            : span(),
          span({ class: 'sign' }, SIGNS[line.kind]),
          code({ class: 'text' }, line.text),
        ),
        inline.length
          ? div({ class: 'inline' }, inline.map((comment) => commentCard(comment, handlers)))
          : null,
      ]
    }
    const renamed = file.oldPath && file.newPath && file.oldPath !== file.newPath
    return section(
      { class: 'file', 'aria-label': file.path },
      header(
        { class: 'head' },
        strong({ class: 'path' }, renamed ? `${file.oldPath} -> ${file.newPath}` : file.path),
        span({ class: 'status' }, STATUS_TEXT[file.status]),
        span({ class: 'added-count' }, `+${file.additions}`),
        span({ class: 'deleted-count' }, `-${file.deletions}`),
        canComment
          ? button(
            {
              class: 'small-button',
              events: { click: () => onComment({ target: fileTarget(file.path), quote: null }) },
            },
            'Comment on file',
          )
          : null,
      ),
      fileComments.length
        ? div(
          { class: 'file-comments' },
          fileComments.map((comment) => commentCard(comment, handlers)),
        )
        : null,
      file.binary
        ? p({ class: 'note' }, 'A binary file. todoer does not show its content.')
        : file.hunks.length === 0
        ? p({ class: 'note' }, renamed ? 'The content is the same.' : 'No change in the content.')
        : div(
          { class: 'lines' },
          file.hunks.map((hunk) => [
            div({ class: 'hunk' }, hunk.header),
            hunk.lines.map(lineRow),
          ]),
        ),
    )
  },
})

export const commentList = component('todoer-comment-list', {
  styles: `
    :host { display: block; }
    .list { display: flex; flex-direction: column; gap: var(--imp-space-2, 8px); }
    ${COMMENT_STYLES}
  `,
  setup: (_self, { comments, editable, onEdit, onDelete }) =>
    div(
      { class: 'list' },
      comments.map((comment) => commentCard(comment, { editable, onEdit, onDelete })),
    ),
})
