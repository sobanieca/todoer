import { component } from 'imp'
import { div } from 'imp/html'

export const page = component('todoer-page', {
  styles: `
    :host { display: block; }
    .page {
      display: flex;
      flex-direction: column;
      gap: var(--imp-space-5, 24px);
      padding: var(--imp-space-page, 24px);
      max-inline-size: 90rem;
      margin-inline: auto;
    }
  `,
  setup: (_self, ...children) => div({ class: 'page' }, children),
})
