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
  | "NoToken"
  | {
      /**
       * @minItems 2
       * @maxItems 2
       */
      AuthSuccess: [User, UserToken];
    }
  | {
      /**
       * @minItems 2
       * @maxItems 2
       */
      AuthSuccessNewPin: [User, UserToken];
    };
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "WebsocketPublish".
 */
export type WebsocketPublish =
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
      data: null;
      path: "user/logout";
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
    };
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "WebsocketRpcResponse".
 */
export type WebsocketRpcResponse =
  | {
      data: AuthResult;
      path: "user/auth_with_token";
    }
  | {
      data: AuthResult;
      path: "user/auth_with_pin";
    }
  | {
      data: null;
      path: "user/logout";
    }
  | {
      data: string;
      path: "debug/test_endpoint";
    }
  | {
      data: null;
      path: "arena/signal";
    };

export interface TempWebsocketRootSchema {
  [k: string]: unknown;
}
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "User".
 */
export interface User {
  permissions: string[];
  pin_hash?: string | null;
  pin_is_numeric: boolean;
  realname: string;
  tokens: string[];
  username: string;
}
/**
 * This interface was referenced by `TempWebsocketRootSchema`'s JSON-Schema
 * via the `definition` "UserToken".
 */
export interface UserToken {
  token: string;
  user: string;
}
