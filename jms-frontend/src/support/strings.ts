export function nullIfEmpty(s: string | null | undefined) : string|null {
  if (s === "" || s === null || s === undefined || ((typeof s) === "string") && s.trim() === "")
    return null;
  return s;
}