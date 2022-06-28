export function withVal<T,R>(val: T, fn: (v: T|null|undefined) => R|null) {
  if (val !== null && val !== undefined)
    return fn(val)
  return null
}

export type Combine<A, B> = A & Omit<B, keyof A>;