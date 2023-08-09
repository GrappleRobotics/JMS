'use client';
import { v4 as uuid } from 'uuid';
import { User, UserToken, WebsocketPublish, WebsocketRpcRequest, WebsocketRpcResponse } from '../ws-schema';
import { KeysOfUnion } from './util';

export type ConnectCallback = (isOpen: boolean) => void;

export function jmsAuthToken(): UserToken | null {
  let token = localStorage.getItem("jmsAuthToken");
  if (token !== null) {
    return JSON.parse(token);
  } else {
    return null;
  }
}

export function clearJmsAuthToken() {
  localStorage.removeItem("jmsAuthToken");
}

export function setJmsAuthToken(token: UserToken) {
  localStorage.setItem("jmsAuthToken", JSON.stringify(token));
}

export default class JmsWebsocket {
  timeout: number;
  ws?: WebSocket;
  connectCallbacks: Map<string, ConnectCallback>;
  callbacks: Map<string, { path: string, fn: (msg: any) => void }>;
  reply_waiting: Map<string, { accept: (msg: any) => void, reject: (error: string) => void }>;
  on_login: Map<string, (token: User | null) => void>;
  user: User | null;
  send_queue: object[];

  constructor(timeout=250) {
    this.timeout = timeout;
    this.callbacks = new Map();
    this.connectCallbacks = new Map();
    this.reply_waiting = new Map();
    this.on_login = new Map();
    this.user = null;
    this.send_queue = [];

    this.connect = this.connect.bind(this);
    this.dead = this.dead.bind(this);
    this.alive = this.alive.bind(this);
    this.tryReconnect = this.tryReconnect.bind(this);
    this.call = this.call.bind(this);
    this.subscribe = this.subscribe.bind(this);
    this.onConnectChange = this.onConnectChange.bind(this);
    this.logout = this.logout.bind(this);
    this.login = this.login.bind(this);
    this.send = this.send.bind(this);
    this.sendNow = this.sendNow.bind(this);
  }

  connect(url: string) {
    let that = this;
    var timer: any;
    var ws = new WebSocket(url);

    ws.onopen = () => {
      console.log("WS Connected");
      setTimeout(() => {
        setTimeout(() => {
          this.connectCallbacks.forEach(cb => cb(true));
          this.callbacks.forEach(cb => this.sendNow({
            message_id: uuid(),
            path: "subscribe",
            data: cb.path
          }));

          for (let msg of this.send_queue) {
            this.sendNow(msg);
          }

          /* Try to login */
          this.call<"user/auth_with_token">("user/auth_with_token", null)
            .then(result => {
              if (result["type"] === "AuthSuccess" || result["type"] === "AuthSuccessNewPin") {
                setJmsAuthToken(result["token"]);

                if (result["type"] === "AuthSuccessNewPin") {
                  let key = prompt(`Please set a new PIN for User: ${result["user"].username}`)
                  if (key !== null) {
                    this.call<"user/update_pin">("user/update_pin", { pin: key })
                      .then(user => {
                        this.on_login.forEach(cb => cb(user));
                        this.user = result["user"];
                      })
                      .catch(e => alert(`Error! ${e}`))
                  }
                } else {
                  this.on_login.forEach(cb => cb(result["user"]))
                  this.user = result["user"];
                }
              }
            })
            .catch(error => clearJmsAuthToken())
        }, 100);
      }, 100);
      that.ws = ws;
      clearTimeout(timer);
    };

    ws.onclose = e => {
      console.log("WS Closed, retrying...", e.reason);
      this.connectCallbacks.forEach(cb => cb(false));
      this.user = null;
      this.on_login.forEach(cb => cb(null));
      timer = setTimeout(() => that.tryReconnect(url), that.timeout);
    };

    ws.onerror = err => {
      console.log("WS Error, closing...", err);
      this.connectCallbacks.forEach(cb => cb(false));
      this.user = null;
      this.on_login.forEach(cb => cb(null));
      ws.close();
    };

    ws.onmessage = msg => {
      let meta = JSON.parse(msg.data);
      let data = meta.data;

      if (meta.replying_to != null) {
        // Reconcile Reply
        const waiting = this.reply_waiting.get(meta.replying_to);
        if (waiting != null) {
          if (meta.error != null) {
            waiting.reject(meta.error);
          } else {
            waiting.accept(data);
          }
          this.reply_waiting.delete(meta.reply);
        } else {
          console.warn(`Got a reply for Message ID ${meta.replying_to} but there are no waiting promises!`);
        }
      } else if (meta.path === "ping") {
        this.send({
          message_id: uuid(),
          replying_to: meta.message_id,
          path: "pong"
        });
      } else {
        this.callbacks.forEach(cb => {
          if (meta.path === `${cb.path}`) {
            cb.fn(data);
          }
        });
      }
    };
  }

  close() {
    this.ws?.close();
  }

  dead() {
    return this.ws == undefined || this.ws.readyState === WebSocket.CLOSED;
  }

  alive() {
    return !this.dead();
  }

  tryReconnect(url: string) {
    if (this.dead())
      this.connect(url);
  }

  call = <Path extends WebsocketRpcRequest["path"]>
         (path: Path, args: Extract<WebsocketRpcRequest, { path: Path }>["data"]): Promise<Extract<WebsocketRpcResponse, { path: Path }>["data"]> => 
  {
    let msg_id = uuid();
    let that = this;
    let p = new Promise<Extract<WebsocketRpcResponse, { path: Path }>["data"]>((resolve, reject) => {
      that.reply_waiting.set(msg_id, {
        accept: (msg: any) => resolve(msg as any),
        reject: reject
      })
    });

    this.send({
      message_id: msg_id,
      path: path,
      data: args,
      token: jmsAuthToken()
    });

    return p as any;
  }
  
  subscribe = <Path extends WebsocketPublish["path"]>
              (path: Path, fn: (data: Extract<WebsocketPublish, { path: Path }>["data"]) => void) => 
  {
    let callback_id = uuid();
    this.callbacks.set(callback_id, { path: path as string, fn: (msg: any) => fn(msg) });

    if (this.alive()) {
      this.sendNow({
        message_id: uuid(),
        path: "subscribe",
        data: path
      });
    }

    return callback_id;
  }

  send(data: any) {
    if (this.alive()) {
      this.sendNow(data);
    } else {
      this.send_queue.push(data);
    }
  }

  sendNow(data: any) {
    this.ws!.send(JSON.stringify(data));
  }

  removeHandle(id: string) {
    if (this.callbacks.has(id))
      this.callbacks.delete(id);
    if (this.connectCallbacks.has(id))
      this.connectCallbacks.delete(id);
    if (this.on_login.has(id))
      this.on_login.delete(id);
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

  onLogin(cb: (user: User | null) => void): string {
    let id = uuid();
    this.on_login.set(id, cb);
    cb(this.user);
    return id;
  }

  logout() {
    this.call<"user/logout">("user/logout", null)
      .then(v => {
        this.user = null;
        clearJmsAuthToken();
        this.on_login.forEach(cb => cb(null));
      });
  }

  login(username: string, pin: string): Promise<void> {
    let p = new Promise<void>((resolve, reject) => {
      this.call<"user/auth_with_pin">("user/auth_with_pin", { username, pin })
        .then(result => {
            if (result["type"] === "AuthSuccess" || result["type"] === "AuthSuccessNewPin") {
              setJmsAuthToken(result["token"]);

              if (result["type"] === "AuthSuccessNewPin") {
                let key = prompt(`Please set a new PIN for User: ${result["user"].username}`)
                if (key !== null) {
                  this.call<"user/update_pin">("user/update_pin", { pin: key })
                    .then(user => {
                      this.on_login.forEach(cb => cb(user));
                      this.user = result["user"];
                    })
                    .catch(e => alert(`Error! ${e}`))
                }
              } else {
                this.on_login.forEach(cb => cb(result["user"]))
                this.user = result["user"];
              }
            }
          resolve();
        })
        .catch(e => {
          clearJmsAuthToken();
          reject(e);
        });
    });
    return p;
  }
}