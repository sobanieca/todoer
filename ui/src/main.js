import { init, mount } from 'imp'
import { root } from './app.js'

await init({ theme: 'imp-light' })

mount('#app', root())
