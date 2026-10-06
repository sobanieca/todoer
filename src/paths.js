export const join = (...parts) =>
  parts.filter((part) => part !== '').join('/').replace(/\/{2,}/g, '/')

export const dirname = (path) => {
  const trimmed = path.length > 1 ? path.replace(/\/+$/, '') : path
  const index = trimmed.lastIndexOf('/')
  if (index === -1) return '.'
  if (index === 0) return '/'
  return trimmed.slice(0, index)
}

export const getHome = () => Deno.env.get('HOME') || Deno.env.get('USERPROFILE') || '.'

export const expandHome = (path) => {
  if (path === '~') return getHome()
  return path.startsWith('~/') ? join(getHome(), path.slice(2)) : path
}

export const absolutePath = (path) => {
  const expanded = expandHome(path)
  return expanded.startsWith('/') ? expanded : join(Deno.cwd(), expanded)
}
