export default class JmsWebsocket {
  constructor(url="ws://" + window.location.hostname + ":9000", timeout=250) {
    this.url = url;
    this.timeout = timeout;
    this.callbacks = [];
    this.connectCallbacks = [];
    this.errorCallbacks = [];

    this.ws = null;

    this.connect = this.connect.bind(this);
    this.dead = this.dead.bind(this);
    this.alive = this.alive.bind(this);
    this.tryReconnect = this.tryReconnect.bind(this);
    this.send = this.send.bind(this);
    this.onMessage = this.onMessage.bind(this);
    this.onConnectChange = this.onConnectChange.bind(this);
    this.onError = this.onError.bind(this);
  }

  connect() {
    let that = this;
    var timer;
    var ws = new WebSocket(this.url);

    ws.onopen = () => {
      console.log("WS Connected");
      setTimeout(() => {
        this.connectCallbacks.forEach(cb => cb(true));
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
      let message = JSON.parse(msg.data);

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

  send(obj, noun, verb, data=null) {
    if (this.alive()) {
      let msg = {
        object: obj, noun: noun, verb: verb, data: data
      };
      this.ws.send(JSON.stringify(msg));
    } else {
      console.log("Can't send message, WS dead :X", noun, verb, data);
    }
  }

  onMessage(obj, noun, verb, cb) {
    this.callbacks.push({o: obj, n: noun, v: verb, c: cb});
  }

  onConnectChange(cb) {
    this.connectCallbacks.push(cb);
  }

  onError(cb) {
    this.errorCallbacks.push(cb);
  }
}