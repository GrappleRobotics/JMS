import { WebsocketMessage2JMS, WebsocketMessage2UI } from "ws-schema";
import { v4 as uuid } from 'uuid';

export type CallbackFn<T> = (msg: T, fullMessage: WebsocketMessage2UI) => void;
export type ConnectCallback = (isOpen: boolean) => void;
type Callback<T> = {
  path: string[],
  fn: CallbackFn<T>
};

export default class JmsWebsocket {
  url: string;
  timeout: number;
  ws?: WebSocket;
  connectCallbacks: Map<string, ConnectCallback>;
  callbacks: Map<string, Callback<any>>;

  constructor(url="ws://" + window.location.hostname + ":9000", timeout=250) {
    this.url = url;
    this.timeout = timeout;
    this.callbacks = new Map<string, Callback<any>>();
    this.connectCallbacks = new Map<string, ConnectCallback>();

    this.connect = this.connect.bind(this);
    this.dead = this.dead.bind(this);
    this.alive = this.alive.bind(this);
    this.tryReconnect = this.tryReconnect.bind(this);
    this.send = this.send.bind(this);
    this.onMessage = this.onMessage.bind(this);
    this.onConnectChange = this.onConnectChange.bind(this);
  }

  connect() {
    let that = this;
    var timer: any;
    var ws = new WebSocket(this.url);

    ws.onopen = () => {
      console.log("WS Connected");
      setTimeout(() => {
        this.connectCallbacks.forEach(cb => cb(true));
        this.callbacks.forEach(cb => this.send({ Subscribe: cb.path }));
      }, 100);
      that.ws = ws;
      clearTimeout(timer);
    };

    ws.onclose = e => {
      console.log("WS Closed, retrying...", e.reason);
      this.connectCallbacks.forEach(cb => cb(false));
      timer = setTimeout(that.tryReconnect, that.timeout);
    };

    ws.onerror = err => {
      console.log("WS Error, closing...", err);
      this.connectCallbacks.forEach(cb => cb(false));
      ws.close();
    };

    ws.onmessage = msg => {
      if (msg.data !== "ping") {
        let messages = JSON.parse(msg.data) as WebsocketMessage2UI[];

        messages.forEach(message => {
          this.callbacks.forEach(cb => {
            // Check variant tree
            let path = cb.path;
            let valid = true;
            let child_msg: any = message;
            for (let i = 0; i < path.length && valid; i++) {
              if (path[i] in child_msg) {
                child_msg = child_msg[path[i]];
              } else if (path[i] === child_msg) {
                // Special case for unit variants
                child_msg = {};
              } else {
                valid = false;
              }
            }

            // Dispatch
            if (valid) {
              cb.fn(child_msg, message);
            }
          });
        });
      }
    };
  }

  close() {
    this.ws?.close();
  }

  dead() {
    return !this.ws || this.ws.readyState === WebSocket.CLOSED;
  }

  alive() {
    return !this.dead();
  }

  tryReconnect() {
    if (this.dead())
      this.connect();
  }

  send(msg: WebsocketMessage2JMS) {
    if (this.alive()) {
      this.ws!.send(JSON.stringify(msg));
    } else {
      console.log("Can't send message, WS dead :X", msg);
    }
  }

  onMessage<T>(path: string[], callback: CallbackFn<T>): string {
    let id = uuid();
    this.callbacks.set(id, {
      path: path,
      fn: callback
    });

    if (this.alive()) {
      this.send({ Subscribe: path });
    }

    return id;
  }

  removeHandle(id: string) {
    if (this.callbacks.has(id))
      this.callbacks.delete(id);
    if (this.connectCallbacks.has(id))
      this.connectCallbacks.delete(id);
  }

  removeHandles(ids?: string[]) {
    // Sometimes set as undefined when live-reloading in dev
    if (ids !== undefined && this !== undefined && this.removeHandle !== undefined)
      ids.forEach(this.removeHandle);
  }

  onConnectChange(cb: ConnectCallback): string {
    let id = uuid();
    this.connectCallbacks.set(id, cb);
    cb(this.alive());
    return id;
  }
}