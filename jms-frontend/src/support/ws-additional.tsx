import { Alliance, NearFar, ResourceRole } from "ws-schema";
import { capitalise } from "./strings";

export const ALLIANCES: Alliance[] = [ "red", "blue" ];
export const ALLIANCE_STATIONS: number[] = [ 1, 2, 3 ];
export const NEAR_FAR: NearFar[] = [ "near", "far" ];

export const ROLES: ResourceRole[] = [
  "Any",
  "Unknown",
  "ScorekeeperPanel",
  "MonitorPanel",
  "TimerPanel",
  "AudienceDisplay",
  "FieldElectronics",
  { RefereePanel: "HeadReferee" },
  ...(ALLIANCES.flatMap(a => NEAR_FAR.map(nf => ({ RefereePanel: { Alliance: [a, nf] as [Alliance, NearFar] } })))),
  ...(ALLIANCES.flatMap(a => ALLIANCE_STATIONS.map(stn => ({ TeamEStop: { alliance: a, station: stn } })))),
  { ScorerPanel: { goals: "AB", height: "low" } },
  { ScorerPanel: { goals: "AB", height: "high" } },
  { ScorerPanel: { goals: "CD", height: "low" } },
  { ScorerPanel: { goals: "CD", height: "high" } },
];

export function role2string(role: ResourceRole) {
  if (typeof role === "object") {
    if ("RefereePanel" in role) {
      if (role.RefereePanel === "HeadReferee")
        return "Referee: Head Ref";
      else 
        return `Referee: ${capitalise(role.RefereePanel.Alliance[0])} ${capitalise(role.RefereePanel.Alliance[1])}`;
    } else if ("ScorerPanel" in role) {
      return `Scorer: ${role.ScorerPanel.goals}${role.ScorerPanel.height[0].toUpperCase()}`
    } else if ("TeamEStop" in role) {
      return `EStop: ${capitalise(role.TeamEStop.alliance)} ${role.TeamEStop.station}`
    }
    return JSON.stringify(role);
  } else {
    const ROLE_LABEL_MAP: { [k in typeof role]: string } = {
      "Unknown": "Other",
      "Any": "Any",
      "ScorekeeperPanel": "Scorekeeper",
      "MonitorPanel": "Monitor",
      "TimerPanel": "Timer",
      "AudienceDisplay": "Audience Display",
      "FieldElectronics": "Field Electronics",
    }
  
    return ROLE_LABEL_MAP[role];
  }
}

export function role2id(role: ResourceRole): string {
  if (typeof role === "object") {
    if ("RefereePanel" in role) {
      if (role.RefereePanel === "HeadReferee")
        return "ref-head";
      else 
        return `ref-${role.RefereePanel.Alliance[0]}-${role.RefereePanel.Alliance[1]}`;
    } else if ("ScorerPanel" in role) {
      return `scorer-${role.ScorerPanel.goals}${role.ScorerPanel.height[0]}`
    } else if ("TeamEStop" in role) {
      return `estop-${role.TeamEStop.alliance}-${role.TeamEStop.station}`
    }
    return JSON.stringify(role);
  } else {
    const ROLE_LABEL_MAP: { [k in typeof role]: string } = {
      "Unknown": "unknown",
      "Any": "any",
      "ScorekeeperPanel": "scorekeeper",
      "MonitorPanel": "monitor",
      "TimerPanel": "timer",
      "AudienceDisplay": "audience-display",
      "FieldElectronics": "field-electronics",
    }
  
    return ROLE_LABEL_MAP[role];
  }
}
