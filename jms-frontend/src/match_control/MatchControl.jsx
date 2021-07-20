import Alliance from "./Alliance";
import MatchFlow from "./MatchFlow";
import React from "react";
import { Button, Col, Container, Row } from "react-bootstrap";
import MatchScheduleView from "./MatchScheduleView";

export default class MatchControl extends React.Component {
  render() {
    let { arena, matches, ws } = this.props;

    return <Container>
      <Row>
        <Col>
          <h3> { arena?.match?.match?.name || <i>No Match Loaded</i> } </h3>
        </Col>
        <Col md="auto">
          <Button
            variant="warning"
            onClick={() => ws.send("arena", "match", "loadTest")}
            disabled={arena?.state?.state !== "Idle"}
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
                state={arena?.state}
                stations={arena?.stations?.filter(x => x.station.alliance === "Blue")}
                onStationUpdate={ (data) => ws.send("arena", "alliances", "update", data) }
              />
            </Col>
            <Col>
              <Alliance
                colour="Red"
                state={arena?.state}
                stations={arena?.stations?.filter(x => x.station.alliance === "Red").reverse()}  // Red teams go 3-2-1 to order how they're seen from the scoring table
                onStationUpdate={ (data) => ws.send("arena", "alliances", "update", data) }
              />
            </Col>
          </Row>
          <br />
          <MatchFlow
            state={arena?.state}
            match={arena?.match}
            onSignal={(data) => ws.send("arena", "state", "signal", data)}
          />
        </Col>
      </Row>
      <br />
      <Row>
        <Col>
          <MatchScheduleView
            arena={arena}
            matches={matches}
            onLoad={(match) => ws.send("arena", "match", "load", match.id)}
          />
        </Col>
      </Row>
    </Container>
  }
}