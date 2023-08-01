/* eslint-disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "ArenaSignal".
 */
export type ArenaSignal =
  | ("Estop" | "EstopReset" | "Prestart" | "PrestartUndo" | "MatchPlay" | "MatchCommit")
  | {
      MatchArm: {
        force: boolean;
      };
    };
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "ArenaState".
 */
export type ArenaState =
  | {
      state: "Init";
    }
  | {
      state: "Reset";
    }
  | {
      net_ready: boolean;
      state: "Idle";
    }
  | {
      state: "Estop";
    }
  | {
      net_ready: boolean;
      state: "Prestart";
    }
  | {
      state: "MatchArmed";
    }
  | {
      state: "MatchPlay";
    }
  | {
      net_ready: boolean;
      state: "MatchComplete";
    };
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "AuthResult".
 */
export type AuthResult =
  | {
      token: UserToken;
      type: "AuthSuccess";
      user: User;
    }
  | {
      token: UserToken;
      type: "AuthSuccessNewPin";
      user: User;
    }
  | {
      type: "NoToken";
    };
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "Permission".
 */
export type Permission = "Admin" | "FTA" | "FTAA" | "ManageEvent" | "ManageTeams" | "ManageSchedule";
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "ScheduleBlockType".
 */
export type ScheduleBlockType = "General" | "Qualification" | "Playoff";
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "WebsocketPublish".
 */
export type WebsocketPublish =
  | {
      data: EventDetails;
      path: "event/details";
    }
  | {
      data: Team[];
      path: "team/teams";
    }
  | {
      data: string;
      path: "debug/test_publish";
    }
  | {
      data: ArenaState;
      path: "arena/state";
    };
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "WebsocketRpcRequest".
 */
export type WebsocketRpcRequest =
  | {
      data: {
        details: EventDetails;
      };
      path: "event/update";
    }
  | {
      data: null;
      path: "event/schedule_get";
    }
  | {
      data: {
        team: Team;
      };
      path: "team/update";
    }
  | {
      data: {
        team_number: number;
      };
      path: "team/delete";
    }
  | {
      data: {
        in_text: string;
      };
      path: "debug/test_endpoint";
    }
  | {
      data: {
        signal: ArenaSignal;
      };
      path: "arena/signal";
    }
  | {
      data: null;
      path: "user/auth_with_token";
    }
  | {
      data: {
        pin: string;
        username: string;
      };
      path: "user/auth_with_pin";
    }
  | {
      data: {
        pin: string;
      };
      path: "user/update_pin";
    }
  | {
      data: null;
      path: "user/logout";
    }
  | {
      data: null;
      path: "user/users";
    }
  | {
      data: {
        user: User;
      };
      path: "user/modify_user";
    }
  | {
      data: {
        user_id: string;
      };
      path: "user/delete_user";
    };
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "WebsocketRpcResponse".
 */
export type WebsocketRpcResponse =
  | {
      data: EventDetails;
      path: "event/update";
    }
  | {
      data: ScheduleBlock[];
      path: "event/schedule_get";
    }
  | {
      data: Team;
      path: "team/update";
    }
  | {
      data: null;
      path: "team/delete";
    }
  | {
      data: string;
      path: "debug/test_endpoint";
    }
  | {
      data: null;
      path: "arena/signal";
    }
  | {
      data: AuthResult;
      path: "user/auth_with_token";
    }
  | {
      data: AuthResult;
      path: "user/auth_with_pin";
    }
  | {
      data: User;
      path: "user/update_pin";
    }
  | {
      data: null;
      path: "user/logout";
    }
  | {
      data: User[];
      path: "user/users";
    }
  | {
      data: null;
      path: "user/modify_user";
    }
  | {
      data: null;
      path: "user/delete_user";
    };

export interface TempWebsocketRootSchema {
  [k: string]: unknown;
}
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "UserToken".
 */
export interface UserToken {
  token: string;
  user: string;
}
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "User".
 */
export interface User {
  permissions: Permission[];
  pin_hash?: string | null;
  pin_is_numeric: boolean;
  realname: string;
  tokens: string[];
  username: string;
}
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "EventDetails".
 */
export interface EventDetails {
  av_chroma_key: string;
  av_event_colour: string;
  code?: string | null;
  event_name?: string | null;
  webcasts: string[];
}
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "ScheduleBlock".
 */
export interface ScheduleBlock {
  block_type: ScheduleBlockType;
  cycle_time: number;
  end_time: string;
  id: string;
  name: string;
  start_time: string;
}
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "Team".
 */
export interface Team {
  affiliation?: string | null;
  display_number: string;
  location?: string | null;
  name?: string | null;
  notes?: string | null;
  number: number;
  schedule: boolean;
  wpakey?: string | null;
}
