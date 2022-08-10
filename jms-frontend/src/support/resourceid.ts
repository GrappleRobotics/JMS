import { v4 as uuid } from 'uuid';

export default function resource_id() {
  const id = sessionStorage.getItem("resource_id");
  if (id == null) {
    sessionStorage.setItem("resource_id", uuid());
  }
  return sessionStorage.getItem("resource_id")!;
}
