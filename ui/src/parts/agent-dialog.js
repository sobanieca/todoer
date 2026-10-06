import { component } from 'imp'
import { useHttpClient } from 'imp/http'
import { initState, useState } from 'imp/state'
import { button, text } from 'imp/std'
import { notice, notify } from 'imp/std/feedback'
import { segmented } from 'imp/std/inputs'
import { dialog, flex, stack } from 'imp/std/layout'
import { messageOf } from '../lib/api.js'
import { promptBox } from './prompt-box.js'

const MODES = [
  { value: 'together', label: 'One worktree for all' },
  { value: 'each', label: 'One worktree each' },
]

const agentBody = component('todoer-agent-body', {
  setup: (self, { request, open }) => {
    const api = useHttpClient(self, 'todoer.api')
    const project = useState(self, 'todoer.project')
    const mode = initState(self, 'todoer.agent.mode', 'together')
    const prompts = initState(self, 'todoer.agent.prompts', undefined)
    const error = initState(self, 'todoer.agent.error', null)
    const running = initState(self, 'todoer.agent.running', false)
    const payload =
      () => (request.pr ? { pr: request.pr } : { todos: request.todos, mode: mode.get() })
    const load = async () => {
      try {
        const answer = await api.post('/agent/prompts', { body: payload() })
        if (self.abortSignal.aborted) return
        prompts.set(answer.prompts)
        error.set(null)
      } catch (failure) {
        error.set(messageOf(failure))
      }
    }
    mode.watch(self, load)
    load()
    const run = async () => {
      running.set(true)
      try {
        const { runs } = await api.post('/agent/run', { body: payload() })
        notify({
          kind: 'success',
          title: runs.length === 1 ? 'The agent started' : `${runs.length} agents started`,
          description: `The output goes to ${runs.map((entry) => entry.log).join(', ')}`,
        })
        open.set(false)
      } catch (failure) {
        error.set(messageOf(failure))
      } finally {
        running.set(false)
      }
    }
    const command = project.get().agentCommand
    return stack(
      { gap: 4 },
      request.todos?.length > 1
        ? segmented({ label: 'Worktrees', value: mode, options: MODES })
        : null,
      text(
        { tone: 'secondary', size: 'sm' },
        'Give this prompt to your coding agent in this project. The agent reads the workflow ' +
          'with "todoer skill", works in a new worktree and sends a pull request.',
      ),
      prompts.view((list) =>
        list === undefined ? null : stack(
          { gap: 3 },
          ...list.map((prompt, index) =>
            promptBox({ prompt, label: list.length > 1 ? `Prompt ${index + 1}` : 'Prompt' })
          ),
          list.length > 1
            ? text({ size: 'sm', tone: 'secondary' }, 'Start one agent session for each prompt.')
            : null,
        )
      ),
      error.view((message) => message && notice({ kind: 'error' }, message)),
      command
        ? flex(
          { horizontalAlign: 'end', verticalAlign: 'center' },
          text({ size: 'sm', tone: 'secondary' }, command.join(' ')),
          button({ variant: 'primary', loading: running, events: { click: run } }, 'Run agent'),
        )
        : text(
          { size: 'sm', tone: 'secondary' },
          'To start the agent from this page, set "agentCommand" in todoer.json, ' +
            'like ["claude", "{prompt}"].',
        ),
    )
  },
})

export const agentDialog = component('todoer-agent-dialog', {
  setup: (_self, { open, request }) =>
    dialog(
      { title: 'Ask an agent', open },
      open.view((isOpen) => (isOpen ? agentBody({ request: request(), open }) : null)),
    ),
})
