const ALPHABET = '0123456789abcdefghijklmnopqrstuvwxyz'

export const randomId = (length = 4) => {
  const bytes = crypto.getRandomValues(new Uint8Array(length))
  return [...bytes].map((byte) => ALPHABET[byte % ALPHABET.length]).join('')
}

export const uniqueId = (taken, length = 4) => {
  for (let attempt = 0; attempt < 50; attempt++) {
    const id = randomId(length + Math.floor(attempt / 10))
    if (!taken.has(id) && /[a-z]/.test(id)) return id
  }
  throw new Error('cannot find a free id')
}

export const slugify = (text, max = 40) =>
  text
    .normalize('NFKD')
    .replace(/\p{M}/gu, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, max)
    .replace(/-+$/g, '') || 'item'

export const ID_PATTERN = /^[a-z0-9]{4,8}$/

export const now = () => new Date().toISOString()
