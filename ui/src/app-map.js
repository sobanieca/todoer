import { appMap } from 'imp'

export const app = appMap({
  home: { path: '/', nav: false, description: 'The start page forwards to the todos' },
  todos: {
    description: 'The todos of the project',
    $todoId: {
      label: ({ params }) => params.todoId,
      description: 'One todo: its text, its images and its status',
    },
  },
  groups: {
    description: 'The groups of todos, and the progress of each group',
    $group: {
      label: ({ params }) => params.group,
      description: 'One group: its todos, their order and its pull requests',
    },
  },
  prs: {
    label: 'Pull requests',
    description: 'The pull requests that the agents sent',
    $prId: {
      label: ({ params }) => params.prId,
      description: 'Review one pull request: comment on lines, files and folders',
    },
  },
})
