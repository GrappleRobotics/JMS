import React from "react";
import { WebsocketMessage2JMS } from "ws-schema";
import JmsWebsocket, { CallbackFn } from "./ws";

export type WebsocketContextT = {
  send: (msg: WebsocketMessage2JMS) => void,
  listen: <T>(path: string[], callback: CallbackFn<T>) => string,
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