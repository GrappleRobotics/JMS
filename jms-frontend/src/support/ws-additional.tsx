import { Alliance, NearFar, ResourceRole } from "ws-schema";

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