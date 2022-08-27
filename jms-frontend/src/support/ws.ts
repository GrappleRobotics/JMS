import { RecvMeta, ResourceRole, SendMeta, WebsocketMessage2JMS, WebsocketMessage2UI } from "ws-schema";
import { v4 as uuid } from 'uuid';
import resource_id from "./resourceid";

export type CallbackFn<T> = (msg: T, fullMessage: WebsocketMessage2UI) => void;
export type ConnectCallback = (isOpen: boolean) => void;
type Callback<T> = {
  path: string[],
  fn: CallbackFn<T>
};

export type TransactPromiseV<T> = { msg: T, full: WebsocketMessage2UI };
export type TransactPromise<T> = Promise<TransactPromiseV<T>>;

// Walk down the callback path and message path to get the final message type if applicable
function walkCallback(msg: WebsocketMessage2UI, cb: Callback<any>): boolean {
  // Check variant tree
  let path = cb.path;
  let valid = true;
  let child_msg: any = msg;
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
    cb.fn(child_msg, msg);
    return true;
  }
  return false;
}

const MAX_SEQ_NUM = 65535;

export default class JmsWebsocket {
  url: string;
  timeout: number;
  ws?: WebSocket;
  connectCallbacks: Map<string, ConnectCallback>;
  callbacks: Map<string, Callback<any>>;
  sendQueue: RecvMeta[];
  role: [ResourceRole, string];
  seq_num: number;
  reply_waiting: Map<number, Callback<any> & { reject: (r: any) => void }>;

  constructor(url="ws://" + window.location.hostname + ":9000", timeout=250) {
    this.url = url;
    this.timeout = timeout;
    this.callbacks = new Map<string, Callback<any>>();
    this.connectCallbacks = new Map<string, ConnectCallback>();
    this.sendQueue = [];
    this.role = [ "Unknown", "" ];
    this.seq_num = 0;
    this.reply_waiting = new Map<number, Callback<any> & { reject: (r: any) => void }>();

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
        this.send({ Resource: { SetID: resource_id() } });
        // this.transact<any>({ Resource: { SetID: resource_id() } }, [])
        //   .then(() => {
        //     console.log("WS Ack'd ID");
        //     this.connectCallbacks.forEach(cb => cb(true));
        //     this.callbacks.forEach(cb => this.send({ Subscribe: cb.path }));
        //     this.sendQueue.forEach(sq => this.sendNow(sq));
        //     this.sendQueue = [];
        //     this.send({ Resource: { SetRole: this.role[0] } });
        //   });
        setTimeout(() => {
          this.connectCallbacks.forEach(cb => cb(true));
          this.callbacks.forEach(cb => this.send({ Subscribe: cb.path }));
          this.send({ Subscribe: ["Ping"] });
          this.sendQueue.forEach(sq => this.sendNow(sq));
          this.sendQueue = [];
          this.send({ Resource: { SetRole: this.role[0] } });
        }, 500);
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
      let meta = JSON.parse(msg.data) as SendMeta;
      let message = meta.msg;

      if (meta.reply != null) {
        // Reconcile Reply
        const waiting = this.reply_waiting.get(meta.reply);
        if (waiting != null) {
          if (!walkCallback(message, waiting)) {
            waiting.reject(`Reply callback not assignable to promise - is the path correct?`);
          }
          this.reply_waiting.delete(meta.reply);
        } else {
          console.warn(`Got a reply for SID ${meta.reply} but there are no waiting promises!`);
        }
      } else if (message === "Ping") {
        console.log(message);
        this.send("Pong");
      } else {
        // Trigger all callbacks whom apply
        this.callbacks.forEach(cb => {
          walkCallback(message, cb);
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

  updateRole = (role: ResourceRole, location: string) => {
    if (location !== this.role[1] || this.role[1] === "Unknown") {
      this.role = [role, location];
      this.send({ Resource: { SetRole: role } });
    }
  }

  send(msg: WebsocketMessage2JMS) {
    this.sendMeta({ msg, seq: this.seq_num++ });
    if (this.seq_num >= MAX_SEQ_NUM)
      this.seq_num = 0;
  }

  transact = <T,>(msg: WebsocketMessage2JMS, path?: string[]|string): TransactPromise<T> => {
    const actual_path = path == null ? [] : typeof path === 'string' ? path.split("/") : path;

    let seq = this.seq_num;
    let that = this;
    let p = new Promise<TransactPromiseV<T>>((resolve, reject) => {
      that.reply_waiting.set(seq, { 
        path: actual_path,
        fn: (v: any, full) => resolve({ msg: (v as T), full }),
        reject
      });
    });
    this.send(msg);
    return p;
  }

  sendMeta(meta: RecvMeta) {
    if (this.alive() && this.sendQueue.length === 0) {
      this.sendNow(meta);
    } else {
      // console.log("Can't send message, WS dead :X", msg);
      this.sendQueue.push(meta);
    }
  }
  
  sendNow(meta: RecvMeta) {
    this.ws!.send(JSON.stringify(meta));
  }

  onMessage<T>(path: string[]|string, callback: CallbackFn<T>): string {
    const actual_path = typeof path === 'string' ? path.split("/") : path;

    let id = uuid();
    this.callbacks.set(id, {
      path: actual_path,
      fn: callback
    });

    if (this.alive()) {
      this.send({ Subscribe: actual_path });
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