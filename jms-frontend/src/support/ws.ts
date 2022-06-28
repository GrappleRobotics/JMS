interface Subscription {
  object: string,
  noun: string
}

export interface Message<T> {
  object: string,
  noun: string,
  verb: string,
  data?: T,
  error?: string
}

type MessageCallbackFn<T> = (msg: Message<T>) => void;
interface MessageCallback<T> {
  o: string,
  n: string,
  v: string,
  c: MessageCallbackFn<T>
}

type ConnectCallback = (isOpen: boolean) => void;
type ErrorCallback = (msg: Message<any>) => void;

export default class JmsWebsocket {
  url: string;
  timeout: number;
  callbacks: MessageCallback<any>[];
  connectCallbacks: ConnectCallback[];
  errorCallbacks: ErrorCallback[];
  ws?: WebSocket;
  subscriptions: Subscription[];

  constructor(url="ws://" + window.location.hostname + ":9000", timeout=250) {
    this.url = url;
    this.timeout = timeout;
    this.callbacks = [];
    this.connectCallbacks = [];
    this.errorCallbacks = [];

    this.connect = this.connect.bind(this);
    this.dead = this.dead.bind(this);
    this.alive = this.alive.bind(this);
    this.tryReconnect = this.tryReconnect.bind(this);
    this.send = this.send.bind(this);
    this.onMessage = this.onMessage.bind(this);
    this.onConnectChange = this.onConnectChange.bind(this);
    this.onError = this.onError.bind(this);

    this.subscriptions = [];
  }

  connect() {
    let that = this;
    var timer: any;
    var ws = new WebSocket(this.url);

    ws.onopen = () => {
      console.log("WS Connected");
      setTimeout(() => {
        this.connectCallbacks.forEach(cb => cb(true));
        this.subscriptions.forEach(s => ws.send(JSON.stringify({ object: s.object, noun: s.noun, verb: "__subscribe__" })));
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
        let messages = JSON.parse(msg.data) as Message<any>[];

        messages.forEach(message => {
          if (message.error !== null) {
            console.error("WS Error: ", message);
            this.errorCallbacks.forEach(cb => cb(message));
          } else {
            this.callbacks.forEach(cbobj => {
              let obj_ok = (cbobj.o === "*") || (cbobj.o === message.object);
              let noun_ok = (cbobj.n === "*") || (cbobj.n === message.noun);
              let verb_ok = (cbobj.v === "*") || (cbobj.v === message.verb);
    
              if (obj_ok && noun_ok && verb_ok)
                cbobj.c(message)
            });
          }
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

  send<T>(msg: Message<T>) {
    if (this.alive()) {
      this.ws!.send(JSON.stringify(msg));
    } else {
      console.log("Can't send message, WS dead :X", msg);
    }
  }

  subscribe(obj: string, noun: string) {
    let s = { object: obj, noun: noun };
    if (!this.subscriptions.some(el => el.object === s.object && el.noun === s.noun)) {
      this.subscriptions.push(s);
      if (this.alive()) {
        this.ws!.send(JSON.stringify({ object: obj, noun: noun, verb: "__subscribe__" }));
      }
    } else {
      console.log("Already subscribed: " + JSON.stringify(s));
    }
  }

  onMessage<T>(obj: string, noun: string, verb: string, cb: MessageCallbackFn<T>) {
    this.callbacks.push({o: obj, n: noun, v: verb, c: cb as MessageCallbackFn<any>});
  }

  onConnectChange(cb: ConnectCallback) {
    this.connectCallbacks.push(cb);
  }

  onError(cb: ErrorCallback) {
    this.errorCallbacks.push(cb);
  }
}