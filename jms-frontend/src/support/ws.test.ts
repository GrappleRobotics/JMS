import JmsWebsocket from "./ws";
import WS from "jest-websocket-mock";
import { WebsocketMessage2JMS } from "ws-schema";

describe("websocket behaviour", () => {
  let ws: JmsWebsocket;
  let server: WS;

  beforeEach(() => {
    ws = new JmsWebsocket("ws://localhost:1234");
    WS.clean();
    server = new WS("ws://localhost:1234", { jsonProtocol: true });
    ws.connect();
  });

  test('matches error message', async () => {
    let errfn = jest.fn();
    let arenafn = jest.fn();

    await server.connected;

    ws.onMessage(["Error"], errfn);
    ws.onMessage(["Arena"], arenafn);

    server.send([{ Error: "abcd1234" }]);

    expect(errfn.mock.calls[0]).toEqual(["abcd1234", { Error: 'abcd1234' }]);
    expect(arenafn.mock.calls.length).toBe(0);
  });

  test('subscribes', async () => {
    await server.connected;
    ws.onMessage(["Arena", "State"], jest.fn());
    await expect(server).toReceiveMessage({ Subscribe: [ "Arena", "State" ] })
  });

  test('matches nested', async () => {
    let matchfn = jest.fn();
    let nomatchfn = jest.fn();

    await server.connected;

    ws.onMessage(["Arena", "State"], matchfn);
    ws.onMessage(["Arena", "Alliance"], nomatchfn);

    let msg = { Arena: { State: { Current: { state: "Init" } } } };
    server.send([msg]);

    expect(matchfn.mock.calls[0]).toEqual([ msg.Arena.State, msg ]);
    expect(nomatchfn.mock.calls.length).toBe(0);
  });

  test('generates an appropriate message', async () => {
    let ws2jms: WebsocketMessage2JMS  = {
      Arena: {
        Alliance: {
          UpdateAlliance: {
            station: { alliance: "red", station: 2 },
            team: 5333, astop: false, estop: true, bypass: false
          }
        }
      }
    }

    await server.connected;
    ws.send(ws2jms);

    await expect(server).toReceiveMessage(ws2jms);
  });
});