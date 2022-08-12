import React, { useContext } from "react";
import { useLocation } from "react-router-dom";
import { ResourceRole, WebsocketMessage2JMS, WebsocketMessage2UI } from "ws-schema";
import JmsWebsocket, { CallbackFn, TransactPromise } from "./ws";

export type WebsocketContextT = {
  send: (msg: WebsocketMessage2JMS) => void,
  transact: <T>(msg: WebsocketMessage2JMS, path?: string[]|string) => TransactPromise<T>,
  listen: <T>(path: string|string[], callback: CallbackFn<T>) => string,
  unlisten: (paths: string[]) => void,
  setRole: (role: ResourceRole, location: string) => void,
  connected: boolean
};

export const WebsocketContext = React.createContext<WebsocketContextT>(
  // @ts-ignore
  null
);

export function RoleUpdater(props: { role: ResourceRole, children: React.ReactElement }) {
  let location = useLocation();
  let wsContext = useContext(WebsocketContext);

  React.useEffect(() => {
    wsContext.setRole(props.role, location.pathname);
  }, [location.pathname]);

  return props.children;
}

export function withRole(role: ResourceRole, children: React.ReactElement) {
  return <RoleUpdater role={role}>
    { children }
  </RoleUpdater>
}

export class WebsocketManagerComponent extends React.Component<{ children: React.ReactElement }, WebsocketContextT> {
  socket: JmsWebsocket = new JmsWebsocket();
  handles: string[] = [];

  readonly state: WebsocketContextT = {
    send: this.socket.send,
    transact: this.socket.transact,
    listen: this.socket.onMessage,
    unlisten: this.socket.removeHandles,
    setRole: this.socket.updateRole,
    connected: false
  };

  componentDidMount = () => {
    this.socket.connect();
    this.handles = [
      this.socket.onConnectChange(connected => this.setState({ connected }))
    ]
  }

  componentWillUnmount = () => {
    this.socket.close();
  }

  render() {
    return <WebsocketContext.Provider value={this.state}>
      <RoleUpdater role="Unknown">
        { this.props.children }
      </RoleUpdater>
    </WebsocketContext.Provider>
  }
};

export abstract class WebsocketComponent<P={},S={}> extends React.Component<P,S> {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;

  handles: string[] = [];

  listen = <K extends keyof S>(path: string|string[], key: K) => {
    const fn = (data: S[K]) => {
      this.setState({ [key]: data } as Pick<S, K>)
    };
    return this.listenFn<S[K]>(path, fn);
  }
  
  listenFn = <T,>(path: string|string[], fn: (data: T) => void) => {
    return this.context.listen<T>(path, fn);
  }

  send = (msg: WebsocketMessage2JMS) => this.context.send(msg);
  transact = <T,>(msg: WebsocketMessage2JMS, path?: string[]|string): TransactPromise<T> => this.context.transact<T>(msg, path)

  isConnected = () => this.context.connected;

  componentWillUnmount = () => {
    this.context.unlisten(this.handles)
  };
}