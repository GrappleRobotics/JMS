export function withVal(val, fn) {
  if (val !== null && val !== undefined)
    return fn(val)
  return null
}