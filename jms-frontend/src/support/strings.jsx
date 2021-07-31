export function nullIfEmpty(s) {
  if (s === "" || ((typeof s) === "string") && s.trim() === "")
    return null;
  return s;
}