import { Alert } from "react-bootstrap";
import { Permission, User } from "../ws-schema";
import { useWebsocket } from "./ws-component";
import React from "react";

export const PERMISSIONS: { [k in Permission]: string } = {
  "Admin": "Admin",
  "FTA": "FTA",
  "FTAA": "FTAA"
}

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

export function withPermission<F extends React.ComponentType>(permission: Permission, component: F) : React.ComponentType {
  function WithPermissionsFunc() {
    const { user } = useWebsocket();

    if (!user || !user_has_permission(permission, user)) {
      return <Alert variant="danger"> You don't have permission to access this page! </Alert>
    }

    return React.createElement(component, {}, null);
  }

  return WithPermissionsFunc;
}