import Alliance from "./Alliance";
import MatchFlow from "./MatchFlow";
import React from "react";
import { Button, Col, Container, Row } from "react-bootstrap";

export default class MatchControl extends React.Component {
  render() {
    let { status, ws } = this.props;

    return <Container>
      <Row>
        <Col>
          <h3> { status?.match?.meta?.name || <i>No Match Loaded</i> } </h3>
        </Col>
        <Col md="auto">
          <Button
            variant="warning"
            onClick={() => ws.send("arena", "match", "loadTest")}
            disabled={status?.state?.state !== "Idle"}
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
                state={status?.state}
                stations={status?.alliances?.filter(x => x.station.alliance === "Blue")}
                onStationUpdate={ (data) => ws.send("arena", "alliances", "update", data) }
              />
            </Col>
            <Col>
              <Alliance
                colour="Red"
                state={status?.state}
                stations={status?.alliances?.filter(x => x.station.alliance === "Red").reverse()}  // Red teams go 3-2-1 to order how they're seen from the scoring table
                onStationUpdate={ (data) => ws.send("arena", "alliances", "update", data) }
              />
            </Col>
          </Row>
          <br />
          <MatchFlow
            state={status?.state}
            match={status?.match}
            onSignal={(data) => ws.send("arena", "state", "signal", data)}
          />
        </Col>
      </Row>
    </Container>
  }
}