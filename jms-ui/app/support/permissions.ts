import { Permission, User } from "../ws-schema";

export function has_permission(required: Permission, permission: Permission) {
  if (permission === "Admin") {
    return true;
  }
  return permission === required;
}

export function user_has_permission(required: Permission, user: User) {
  for (let perm of user.permissions) {
    if (has_permission(required, perm)) {
      return true;
    }
  }
  return false;
}

export const PERMISSIONS: { [k in Permission]: string } = {
  "Admin": "Admin",
  "FTA": "FTA",
  "FTAA": "FTAA"
}