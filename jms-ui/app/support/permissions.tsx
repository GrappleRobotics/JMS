import { Alert } from "react-bootstrap";
import { Permission, User } from "../ws-schema";
import { useWebsocket } from "./ws-component";
import React from "react";

export const PERMISSIONS: { [k in Permission]: string } = {
  "Admin": "Admin",
  "FTA": "FTA",
  "FTAA": "FTAA",
  "Scorekeeper": "Scorekeeper",

  "ManageEvent": "Manage Event",
  "ManageTeams": "Manage Teams",
  "ManageSchedule": "Manage Schedule",
  "ManagePlayoffs": "Manage Playoffs",
  "MatchFlow": "Match Flow",
  "ManageAwards": "Manage Awards",
  "Estop": "E-Stop",
  "Scoring": "Scoring",
  "EditScores": "Edit Scores",
  "ManageAlliances": "Manage Alliances",
  "ManageAudience": "Manage Audience Display"
}

// See user.rs in jms-core-lib, this should echo Permission::has
export const PERMISSION_IMPLICATIONS: { [k in Permission]: Permission[] } = {
  "Admin": Object.keys(PERMISSIONS) as Permission[],
  "FTA": [ "ManageEvent", "ManageTeams", "ManageSchedule", "ManagePlayoffs", "ManageAwards", "MatchFlow", "Estop", "ManageAlliances", "ManageAudience" ],
  "FTAA": [ "Estop" ],
  "Scorekeeper": [ "ManageAwards", "MatchFlow", "Estop", "Scoring", "EditScores", "ManageAlliances", "ManageAudience" ],

  /* Implications are not transient, so individual permissions get no implications */
  "ManageEvent": [],
  "ManageTeams": [],
  "ManageSchedule": [],
  "ManagePlayoffs": [],
  "ManageAwards": [],
  "MatchFlow": [],
  "Estop": [],
  "Scoring": [],
  "EditScores": [],
  "ManageAlliances": [],
  "ManageAudience": []
}

export function has_permission(required: Permission, permission: Permission) {
  if (PERMISSION_IMPLICATIONS[permission].includes(required)) {
    return true;
  }
  return permission === required;
}

export function user_has_permission(required: Permission[], user: User) {
  for (let perm of user.permissions) {
    for (let req of required) {
      if (has_permission(req, perm)) {
        return true;
      }
    }
  }
  return false;
}

export function withPermission<P, F extends React.ComponentType<P>>(permissions: Permission[], component: F) : React.ComponentType<P> {
  function WithPermissionsFunc(props: P) {
    const { user } = useWebsocket();

    if (!user || !user_has_permission(permissions, user)) {
      return <Alert variant="danger"> You do not have permission to access this page! </Alert>
    }

    return React.createElement(component as any, props as any, null);
  }

  return WithPermissionsFunc;
}

export function PermissionGate({ children, permissions }: { children: React.ReactNode, permissions: Permission[] }) {
  const { user } = useWebsocket();

  if (!user || !user_has_permission(permissions, user)) {
    return <React.Fragment />
  }

  return children;
}