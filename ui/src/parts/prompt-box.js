import { component } from 'imp'
import { div, p } from 'imp/html'
import { initState } from 'imp/state'
import { button, icon } from 'imp/std'
import { luCheck, luCopy } from 'imp/icons'

export const promptBox = component('todoer-prompt-box', {
  styles: `
    :host { display: block; }
    .box {
      display: flex;
      gap: var(--imp-space-3, 12px);
      align-items: flex-start;
      padding: var(--imp-space-3, 12px);
      border: var(--imp-border-width, 1px) solid var(--imp-color-border, #cbd5e1);
      border-radius: var(--imp-radius-item, 6px);
      background: var(--imp-color-surface-sunken, #f1f5f9);
    }
    .text {
      flex: 1;
      margin: 0;
      font-family: var(--imp-font-family-mono, monospace);
      font-size: var(--imp-font-size-sm, 0.875rem);
      color: var(--imp-color-text, #0f172a);
      white-space: pre-wrap;
      overflow-wrap: anywhere;
    }
  `,
  setup: (self, { prompt, label }) => {
    const copied = initState(self, 'todoer.prompt.copied', false)
    const copy = async () => {
      await navigator.clipboard.writeText(prompt)
      copied.set(true)
      self.timer = setTimeout(() => copied.set(false), 2000)
    }
    return div(
      { class: 'box', role: 'group', 'aria-label': label },
      p({ class: 'text' }, prompt),
      copied.view((done) =>
        button(
          {
            size: 'sm',
            label: done ? 'Copied' : `Copy the ${label.toLowerCase()}`,
            events: { click: copy },
          },
          icon(done ? luCheck : luCopy, { size: 'sm' }),
          done ? 'Copied' : 'Copy',
        )
      ),
    )
  },
  onRemove: (self) => clearTimeout(self.timer),
})
