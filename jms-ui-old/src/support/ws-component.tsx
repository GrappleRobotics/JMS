import React, { useContext } from "react";
import { useLocation } from "react-router-dom";
import JmsWebsocket from "./ws";
import { WebsocketPublish, WebsocketRpcRequest, WebsocketRpcResponse } from "ws-schema";

export type WebsocketContextT = {
  rpc_call:   <Handler extends keyof WebsocketRpcRequest, Method extends keyof WebsocketRpcRequest[Handler]>
              (handler: Handler, method: Method, args: WebsocketRpcRequest[Handler][Method]) => Promise<WebsocketRpcResponse[Handler][Method]>,
  subscribe:  <Handler extends keyof WebsocketPublish, Method extends keyof WebsocketPublish[Handler]>
              (handler: Handler, method: Method, callback: (msg: WebsocketPublish[Handler][Method]) => any) => string,
  unsubscribe: (paths: string[]) => void,
  connected: boolean
};

export const WebsocketContext = React.createContext<WebsocketContextT>(
  // @ts-ignore
  null
);

export class WebsocketManagerComponent extends React.Component<{ children: React.ReactElement }, WebsocketContextT> {
  socket: JmsWebsocket = new JmsWebsocket();
  handles: string[] = [];

  readonly state: WebsocketContextT = {
    rpc_call: this.socket.call,
    subscribe: this.socket.subscribe,
    unsubscribe: this.socket.removeHandles,
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

  subscribe = <Handler extends keyof WebsocketPublish, Method extends keyof WebsocketPublish[Handler], K extends keyof S & WebsocketPublish[Handler][Method]>
          (handler: Handler, method: Method, key: K) => 
  {
    const fn = (data: WebsocketPublish[Handler][Method]) => {
      this.setState({ [key]: data } as Pick<S, K>)
    };
    return this.subscribeFn<Handler, Method>(handler, method, fn);
  }
  
  subscribeFn = <Handler extends keyof WebsocketPublish, Method extends keyof WebsocketPublish[Handler]>
                (handler: Handler, method: Method, fn: (data: WebsocketPublish[Handler][Method]) => void) => {
    let callback_id = this.context.subscribe<Handler, Method>(handler, method, fn);
    this.handles.push(callback_id);
    return callback_id;
  }

  call = <Handler extends keyof WebsocketRpcRequest, Method extends keyof WebsocketRpcRequest[Handler]>
         (handler: Handler, method: Method, args: WebsocketRpcRequest[Handler][Method]): Promise<WebsocketRpcResponse[Handler][Method]> => 
         this.context.rpc_call<Handler, Method>(handler, method, args);

  isConnected = () => this.context.connected;

  componentWillUnmount = () => {
    this.context.unsubscribe(this.handles);
  };
}