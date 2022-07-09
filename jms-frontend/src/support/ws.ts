import { WebsocketMessage2JMS, WebsocketMessage2UI } from "ws-schema";

type CallbackFn<T> = (msg: T, fullMessage: WebsocketMessage2UI) => void;
type ConnectCallback = (isOpen: boolean) => void;
type Callback<T> = {
  path: string[],
  fn: CallbackFn<T>
};

export default class JmsWebsocket {
  url: string;
  timeout: number;
  // callbacks: MessageCallback<any>[];
  // connectCallbacks: ConnectCallback[];
  // errorCallbacks: ErrorCallback[];
  ws?: WebSocket;
  // subscriptions: Subscription[];
  connectCallbacks: ConnectCallback[];
  callbacks: Callback<any>[];

  constructor(url="ws://" + window.location.hostname + ":9000", timeout=250) {
    this.url = url;
    this.timeout = timeout;
    this.callbacks = [];
    this.connectCallbacks = [];

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
              } else {
                valid = false;
              }
            }

            // Dispatch
            if (valid) {
              cb.fn(child_msg, message);
            }
          });

          // if (message.error !== null) {
          //   console.error("WS Error: ", message);
          //   this.errorCallbacks.forEach(cb => cb(message));
          // } else {
          //   this.callbacks.forEach(cbobj => {
          //     let obj_ok = (cbobj.o === "*") || (cbobj.o === message.object);
          //     let noun_ok = (cbobj.n === "*") || (cbobj.n === message.noun);
          //     let verb_ok = (cbobj.v === "*") || (cbobj.v === message.verb);
    
          //     if (obj_ok && noun_ok && verb_ok)
          //       cbobj.c(message)
          //   });
          // }
        });
      }
    };
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

  // subscribe(obj: string, noun: string) {
  //   let s = { object: obj, noun: noun };
  //   if (!this.subscriptions.some(el => el.object === s.object && el.noun === s.noun)) {
  //     this.subscriptions.push(s);
  //     if (this.alive()) {
  //       this.ws!.send(JSON.stringify({ object: obj, noun: noun, verb: "__subscribe__" }));
  //     }
  //   } else {
  //     console.log("Already subscribed: " + JSON.stringify(s));
  //   }
  // }

  onMessage<T>(path: string[], callback: CallbackFn<T>) {
    this.callbacks.push({
      path: path,
      fn: callback
    });

    if (this.alive()) {
      this.send({ Subscribe: path });
    }
  }

  onConnectChange(cb: ConnectCallback) {
    this.connectCallbacks.push(cb);
  }
}