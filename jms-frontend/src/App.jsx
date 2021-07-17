import React from 'react';
import { Route, Switch } from 'react-router-dom';
import JmsWebsocket from 'support/ws';
import Navbar from 'Navbar';
import MatchControl from 'match_control/MatchControl';
import EventWizard from 'wizard/EventWizard';
import { EVENT_WIZARD, MATCH_CONTROL } from 'paths';

export default class App extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      connected: false,
    }

    this.ws = new JmsWebsocket();
    this.ws.onMessage("*", "*", "*", (msg) => {
      this.setState({
        [msg.object]: {
          ...this.state[msg.object],
          [msg.noun]: {
            ...this.state[msg.object]?.[msg.noun],
            [msg.verb]: msg.data
          }
        }
      });
    });

    this.ws.onConnectChange(connected => {
      this.setState({ connected });
    });

    this.ws.onError((err) => {
      alert(err.object + ":" + err.noun + ":" + err.verb + " - " + err.error);
    });

    this.ws.connect();

    window['ws'] = this.ws;

    this.updateInterval = setInterval(() => {
      this.ws.send("arena", "status", "get");
      this.ws.send("event", "details", "get");
      this.ws.send("event", "teams", "get");
      this.ws.send("event", "schedule", "blocks");
    }, 500);
  }

  render() {
    let arena = this.state.arena?.status?.get;

    return <div className="h-100">
      <Navbar
        connected={this.state.connected}
        state={arena?.state}
        match={arena?.match}
        onEstop={() => this.ws.send("arena", "state", "signal", { signal: "Estop" })}
      />

      <br />

      <Switch>
        <Route path={EVENT_WIZARD}>
          <EventWizard
            ws={this.ws}
            event={this.state.event?.details?.get}
            teams={this.state.event?.teams?.get}
            schedule={this.state.event?.schedule}
          />
        </Route>
        <Route path={MATCH_CONTROL}>
          <MatchControl
            ws={this.ws}
            status={arena}
          />
        </Route>
      </Switch>
    </div>
  }
};
