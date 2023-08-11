import { Alliance } from "../ws-schema";

export const ALLIANCES: Alliance[] = [ "blue", "red" ];
export const ALLIANCES_FLIPPED: Alliance[] = [ "red", "blue" ];

export function otherAlliance(alliance: Alliance): Alliance {
  if (alliance === "blue") return "red";
  else return "blue";
}

export const STATIONS: number[] = [1, 2, 3];
export const ALLIANCE_STATIONS: [Alliance, number][] = ALLIANCES.flatMap(a => STATIONS.map(s => [a, s])) as any;