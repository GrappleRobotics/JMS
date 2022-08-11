import { Alliance } from "ws-schema";

export function withVal<T,R>(val: T|null|undefined, fn: (v: T) => R|null) {
  if (val !== null && val !== undefined)
    return fn(val)
  return null
}

export function withValU<T,R>(val: T|null|undefined, fn: (v: T) => R|undefined) {
  if (val !== null && val !== undefined)
    return fn(val)
  return undefined
}

export type Combine<A, B> = A & Omit<B, keyof A>;
export type VariantKeys<T> = T extends any ? keyof T : never;

const inverse_alliance_map: { [K in Alliance]: Alliance } = {
  blue: "red",
  red: "blue"
};

export function otherAlliance(alliance: Alliance): Alliance {
  return inverse_alliance_map[alliance];
}

export function interleave<T>(arr: T[], fn: () => T): T[] {
  return arr.flatMap(n => [n, fn()]).slice(0, -1);
}