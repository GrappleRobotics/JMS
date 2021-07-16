import React from 'react';
import { Route, Switch } from 'react-router-dom';
import JmsWebsocket from 'support/ws';
import Navbar from 'components/navbar';
import MatchControl from 'match_control/MatchControl';

export default class App extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      connected: false,
      status: null
    }

    this.ws = new JmsWebsocket();
    this.ws.onMessage("arena", "status", "get", data => {
      this.setState({ status: data });
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
      if (this.state.connected)
        this.ws.send("arena", "status", "get");
    }, 500);
  }

  render() {
    return <div>
      <Navbar
        connected={this.state.connected}
        state={this.state.status?.state}
        match={this.state.status?.match}
        onEstop={() => this.ws.send("arena", "state", "signal", { signal: "Estop" })}
      />

      <br />

      <Switch>
        <Route path="/">
          <MatchControl
            status={this.state.status}
            ws={this.ws}
          />
        </Route>
      </Switch>
    </div>
  }
};
