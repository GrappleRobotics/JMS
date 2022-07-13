export function nullIfEmpty(s: string | null | undefined) : string|null {
  if (s === "" || s === null || s === undefined || ((typeof s) === "string") && s.trim() === "")
    return null;
  return s;
}

export function undefinedIfEmpty(s: string | null | undefined) : string|undefined {
  return nullIfEmpty(s) || undefined;
}

export function maybeParseInt(s: string | null | undefined) : number | undefined {
  const m = undefinedIfEmpty(s);
  if (m != null)
    return parseInt(m);
  else
    return undefined;
}