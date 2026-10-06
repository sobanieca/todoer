import { component } from 'imp'
import { initHttpClient } from 'imp/http'
import { redirect, router } from 'imp/router'
import { initState } from 'imp/state'
import { icon } from 'imp/std'
import { toast } from 'imp/std/feedback'
import { appLogo, shell, themeSelect } from 'imp/std/shell'
import { luFolderKanban, luGitPullRequest, luListTodo, luSquareCheckBig } from 'imp/icons'
import { app } from './app-map.js'
import { groupScreen } from './screens/group.js'
import { groupsScreen } from './screens/groups.js'
import { prScreen } from './screens/pr.js'
import { prsScreen } from './screens/prs.js'
import { todoScreen } from './screens/todo.js'
import { todosScreen } from './screens/todos.js'

export const root = component('todoer-root', {
  setup: async (self) => {
    const api = initHttpClient(self, 'todoer.api', { baseUrl: '/api', requestId: false })
    const [info, first] = await Promise.all([api.get('/project'), api.get('/graph')])
    if (self.abortSignal.aborted) return null
    const project = initState(self, 'todoer.project', info)
    const revision = initState(self, 'todoer.revision', 0)
    const graph = initState(self, 'todoer.graph', first.nodes)
    revision.watch(self, async () => {
      try {
        const { nodes } = await api.get('/graph')
        if (!self.abortSignal.aborted) graph.set(nodes)
      } catch {
        return
      }
    })
    self.events = new EventSource('/api/events')
    self.events.addEventListener('change', () => revision.set((value) => value + 1))
    return shell(
      {
        logo: appLogo({
          label: 'todoer',
          tagline: project.get().name,
          icon: icon(luSquareCheckBig, { size: 'lg' }),
        }),
        navbar: {
          map: app,
          icons: {
            [app.todos]: luListTodo,
            [app.groups]: luFolderKanban,
            [app.prs]: luGitPullRequest,
          },
        },
        extras: [themeSelect()],
      },
      toast(),
      router({
        map: app,
        screens: {
          [app.home]: redirect(app.todos),
          [app.todos]: () => todosScreen(),
          [app.todos.$todoId]: ({ params }) => todoScreen({ id: params.todoId }),
          [app.groups]: () => groupsScreen(),
          [app.groups.$group]: ({ params }) => groupScreen({ slug: params.group }),
          [app.prs]: () => prsScreen(),
          [app.prs.$prId]: ({ params }) => prScreen({ id: params.prId }),
        },
        title: ({ crumbs }) => `${crumbs.at(-1)} - todoer - ${project.get().name}`,
      }),
    )
  },
  onRemove: (self) => self.events?.close(),
})
