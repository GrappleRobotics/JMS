import { Alliance } from "../ws-schema";

export const ALLIANCES: Alliance[] = [ "blue", "red" ];
export const STATIONS: number[] = [1, 2, 3];
export const ALLIANCE_STATIONS: [Alliance, number][] = ALLIANCES.flatMap(a => STATIONS.map(s => [a, s])) as any;