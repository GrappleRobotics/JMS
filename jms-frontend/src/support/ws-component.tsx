import React from "react";
import { WebsocketMessage2JMS } from "ws-schema";
import JmsWebsocket, { CallbackFn } from "./ws";

export type WebsocketContextT = {
  send: (msg: WebsocketMessage2JMS) => void,
  listen: <T>(path: string|string[], callback: CallbackFn<T>) => string,
  unlisten: (paths: string[]) => void,
  connected: boolean
};

export const WebsocketContext = React.createContext<WebsocketContextT>(
  // @ts-ignore
  null
);

export class WebsocketManagerComponent extends React.Component<{ children: React.ReactNode }, WebsocketContextT> {
  socket: JmsWebsocket = new JmsWebsocket();
  handles: string[] = [];

  readonly state: WebsocketContextT = {
    send: this.socket.send,
    listen: this.socket.onMessage,
    unlisten: this.socket.removeHandles,
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
      { this.props.children }
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

  isConnected = () => this.context.connected;

  componentWillUnmount = () => {
    this.context.unlisten(this.handles)
  };
}