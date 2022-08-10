import { v4 as uuid } from 'uuid';

export default function panel_id() {
  const id = sessionStorage.getItem("panel_id");
  if (id == null) {
    sessionStorage.setItem("panel_id", uuid());
  }
  return sessionStorage.getItem("panel_id")!;
}
