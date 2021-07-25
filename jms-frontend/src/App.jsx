import React from 'react';
import { Route, Switch } from 'react-router-dom';
import JmsWebsocket from 'support/ws';
import MatchControl from 'match_control/MatchControl';
import EventWizard from 'wizard/EventWizard';
import { EVENT_WIZARD, MATCH_CONTROL } from 'paths';
import TopNavbar from 'TopNavbar';
import { Col, Navbar, Row } from 'react-bootstrap';
import BottomNavbar from 'BottomNavbar';
import { nullIfEmpty } from 'support/strings';

export default class App extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      connected: false,
    }

    this.ws = new JmsWebsocket();
    this.ws.onMessage("*", "*", "__update__", msg => {
      if (!!nullIfEmpty(msg.noun)) {
        this.setState({
          [msg.object]: {
            ...this.state[msg.object],
            [msg.noun]: msg.data
          }
        });
      } else {
        this.setState({ [msg.object]: msg.data });
      }
    });

    this.ws.onConnectChange(connected => {
      this.setState({ connected });
    });

    this.ws.onError((err) => {
      alert(err.object + ":" + err.noun + ":" + err.verb + " - " + err.error);
    });

    this.ws.connect();

    window['ws'] = this.ws;
  }

  render() {
    let { arena, event, matches } = this.state;

    return <div className="wrapper">
      <Row>
        <Col>
          <TopNavbar
            connected={this.state.connected}
            state={arena?.state}
            match={arena?.match}
            onEstop={() => this.ws.send("arena", "state", "signal", { signal: "Estop" })}
          />
        </Col>
      </Row>
      <Row className="app-viewport">
        <Col>
          <Switch>
            <Route path={EVENT_WIZARD}>
              <EventWizard
                ws={this.ws}
                event={event}
                matches={matches}
              />
            </Route>
            <Route path={MATCH_CONTROL}>
              <MatchControl
                ws={this.ws}
                arena={arena}
                matches={matches}
              />
            </Route>
          </Switch>
        </Col>
      </Row>
      <Row>
        <Col>
          <BottomNavbar
            arena={arena}
            matches={matches}
            event={event?.details}
          />
        </Col>
      </Row>

    </div>
  }
};
