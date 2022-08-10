import { Alliance } from "ws-schema";

export function withVal<T,R>(val: T|null|undefined, fn: (v: T) => R|null) {
  if (val !== null && val !== undefined)
    return fn(val)
  return null
}

export type Combine<A, B> = A & Omit<B, keyof A>;

const inverse_alliance_map: { [K in Alliance]: Alliance } = {
  blue: "red",
  red: "blue"
};

export function otherAlliance(alliance: Alliance): Alliance {
  return inverse_alliance_map[alliance];
}