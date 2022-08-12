import React from "react";
import { Button, Container, Col, Row, Tabs, Tab, Nav, Form } from "react-bootstrap";
import { capitalise } from "support/strings";
import { WebsocketComponent } from "support/ws-component"
import { MatchStationStatusRecord, MatchStationStatusRecordKey, SerializedMatch, Team } from "ws-schema"
import _ from "lodash"

type HistorianState = {
  matches: SerializedMatch[],
  teams: Team[],
  keys: MatchStationStatusRecordKey[]
}

export default class Historian extends WebsocketComponent<{}, HistorianState> {
  readonly state: HistorianState = {
    matches: [], teams: [], keys: []
  };

  componentDidMount = () => this.handles = [
    this.listenFn<SerializedMatch[]>("Match/All", matches => this.setState({ matches: matches.filter(m => m.played) })),
    this.listen("Event/Team/CurrentAll", "teams"),
    this.listen("Historian/Keys", "keys")
  ]

  render() {
    const { matches } = this.state;

    return <Container fluid className="px-5">
      <Row>
        <Col> <h3> Historian </h3> </Col>
        <Col className="text-end">
          
        </Col>
      </Row>
      <br />
      <Tabs className="historian-tabs" defaultActiveKey="match">
        <Tab eventKey="match" title="Matches">
          <MatchHistorian { ...this.state } />
        </Tab>
        <Tab eventKey="team" title="Teams">

        </Tab>
      </Tabs>
    </Container>
  }
}

type MatchHistorianState = {
  active?: SerializedMatch,
  records: MatchStationStatusRecord[]
};

export class MatchHistorian extends WebsocketComponent<HistorianState, MatchHistorianState> {
  readonly state: MatchHistorianState = { records: [] };

  componentDidUpdate = (prevProps: HistorianState, prevState: MatchHistorianState) => {
    const keys = this.props.keys;
    const active = this.state.active;

    if (this.state.active !== prevState.active) {
      this.setState({ records: [] }, () => {
        if (active != null) {
          console.log(this.state, this.props);
          const matching_keys = keys.filter(k => k.match_id === active.id);
          this.transact<MatchStationStatusRecord[]>({ Historian: { Load: matching_keys } }, "Historian/Load")
              .then(data => this.setState({ records: data.msg }))
        }
      });
    }
  }

  render() {
    const { matches, teams } = this.props;
    const { active, records } = this.state;

    return <Container fluid>
      <Row className="my-3">
        <Col md={3} className="historian-sel-tabs">
          <Nav variant="pills" className="flex-column">
            {
              matches.reverse().map(m => <Nav.Item>
                <Nav.Link active={m.id === active?.id} onClick={() => this.setState({ active: m })}>
                  { m.name }
                </Nav.Link>
              </Nav.Item>)
            }
          </Nav>
        </Col>
        <Col md={9} className="historian-right-col">
          {
            _.sortBy(records, r => `${r.key.station.alliance} ${r.key.station.station}`).map(r => <Row className="historian-row" data-alliance={r.key.station.alliance}>
              <Col md={3}>
                <Row className="station-name">
                  <Col> { r.key.station.alliance.toUpperCase() } { r.key.station.station } &nbsp; &nbsp;{ r.key.team ? `${r.key.team}` : undefined }</Col>
                </Row>
                <Row>
                  { r.record.some(rec => rec.bypass) ? <Col>BYPASSED</Col> : undefined }
                  { r.record.some(rec => rec.estop || rec.ds_report?.estop) ? <Col>E-STOPPED</Col> : undefined }
                </Row>
                <Row className="mt-2">
                  <Col className="text-muted text-start">
                    <p> Avg RTT: { _.mean(r.record.map(rec => rec.ds_report?.rtt || 0)) } </p>
                  </Col>
                </Row>
              </Col>
              <Col md={9}>
              </Col>
            </Row>)
          }
        </Col>
      </Row>
    </Container>
  }
}