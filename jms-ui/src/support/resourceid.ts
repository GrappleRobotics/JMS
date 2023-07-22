import { v4 as uuid } from 'uuid';

// TODO: FTA should persist across tabs - should be in localStorage.

export default function resource_id() {
  const id = sessionStorage.getItem("resource_id");
  if (id == null) {
    sessionStorage.setItem("resource_id", uuid());
  }
  return sessionStorage.getItem("resource_id")!;
}

export function resource_id_lock() {
  // https://stackoverflow.com/questions/56868153/session-storage-not-being-cleared-when-duplicating-tabs
  window.addEventListener('beforeunload', () => sessionStorage.removeItem("__lock"));
  if (sessionStorage.getItem("__lock")) {
    sessionStorage.clear();
    console.warn("Found a lock in session storage - tab must have been duplicated. Clearing storage.");
  }
  sessionStorage.setItem('__lock', 'true');
}

export function get_fta_key() {
  return localStorage.getItem("fta_key")
}

export function set_fta_key(key: string) {
  localStorage.setItem("fta_key", key);
}

export function clear_fta_key() {
  localStorage.removeItem("fta_key");
}