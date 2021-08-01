import React from 'react';
import { Container, Row, Col, Button } from 'react-bootstrap';
import Alliance from './components/alliance';
import MatchFlow from './components/match_flow';
import Navbar from './components/navbar';
import JmsWebsocket from './support/ws';

class App extends React.Component {
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

      <Container>
        <br />
        <Row>
          <Col>
            <h3> { this.state.status?.match?.meta?.name || <i>No Match Loaded</i> } </h3>
          </Col>
          <Col md="auto">
            <Button
              variant="warning"
              onClick={() => this.ws.send("arena", "match", "loadTest")}
              disabled={this.state.status?.state?.state !== "Idle"}
            >
              Load Test Match
            </Button>
          </Col>
        </Row>
        <br />
        <Row >
          <Col>
            <Row>
              <Col>
                <Alliance
                  colour="Blue"
                  state={this.state.status?.state}
                  stations={this.state.status?.alliances?.filter(x => x.station.alliance === "Blue")}
                  onStationUpdate={ (data) => this.ws.send("arena", "alliances", "update", data) }
                />
              </Col>
              <Col>
                <Alliance
                  colour="Red"
                  state={this.state.status?.state}
                  stations={this.state.status?.alliances?.filter(x => x.station.alliance === "Red").reverse()}  // Red teams go 3-2-1 to order how they're seen from the scoring table
                  onStationUpdate={ (data) => this.ws.send("arena", "alliances", "update", data) }
                />
              </Col>
            </Row>
            <br />
            <MatchFlow
              state={this.state.status?.state}
              match={this.state.status?.match}
              onSignal={(data) => this.ws.send("arena", "state", "signal", data)}
            />
          </Col>
        </Row>
      </Container>
    </div>
  }
};

export default App;