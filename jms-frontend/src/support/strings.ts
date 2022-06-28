export function nullIfEmpty(s: string) : string|null {
  if (s === "" || ((typeof s) === "string") && s.trim() === "")
    return null;
  return s;
}