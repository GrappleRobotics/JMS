import { Button, Col, Container, Form, Row } from "react-bootstrap";
import { WebsocketComponent } from "support/ws-component";
import { MatchStationStatusRecordKey } from "ws-schema";
import _ from "lodash";
import { nullIfEmpty } from "support/strings";
import MatchLogView from "./MatchLogView";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCircleNotch, faRotate } from "@fortawesome/free-solid-svg-icons";

type MatchLogsState = {
  keys: MatchStationStatusRecordKey[],
  match_id?: string,
  team?: number,
  loading: boolean,
};

export default class MatchLogs extends WebsocketComponent<{}, MatchLogsState> {
  readonly state: MatchLogsState = { keys: [], loading: false };

  componentDidMount = () => {
    this.loadKeys();
  }
  
  loadKeys = () => {
    this.setState({ loading: true }, () => {
      this.transact<MatchStationStatusRecordKey[]>({
        Ticket: { Logs: "Keys" }
      }, "Ticket/Logs/Keys")
      .then(data => this.setState({ keys: data.msg.reverse(), loading: false }))
    })
  }

  keysFor = (match: string) => {
    return this.state.keys.filter(k => k.match_id === match);
  }

  render() {
    const { match_id, team, loading } = this.state;
    return <Container>
      <Row className="mb-3">
        <Col>
          <h3> Match Logs </h3>
        </Col>
      </Row>
      <Row className="mb-3">
        <Col>
          <Form.Select value={match_id || ""} onChange={v => this.setState({ match_id: nullIfEmpty(v.target.value) || undefined, team: undefined })}>
            <option value=""> Select Match </option>
            {
              _.uniq(this.state.keys.map(k => k.match_id)).map(k => <option value={k}> { k } </option>)
            }
          </Form.Select>
        </Col>
        <Col>
          <Form.Select value={team || ""} onChange={v => this.setState({ team: Number(v.target.value) || undefined })}>
            <option value=""> Select Team </option>
            {
              this.keysFor(match_id || "").map(k => <option value={k.team}> { k.team } </option>)
            }
          </Form.Select>
        </Col>
        <Col md="auto">
          <Button disabled={loading} onClick={this.loadKeys}>
            <FontAwesomeIcon icon={loading ? faCircleNotch : faRotate} spin={loading} />
            &nbsp; Refresh
          </Button>
        </Col>
      </Row>
      <Row>
        <Col>
            {
              (match_id == null || team == null) ? <h5 className="text-muted"> Select a Match and Team to Load Logs </h5>
                : <MatchLogView autoload match_id={match_id!} team={team!} />
            }
        </Col>
      </Row>
    </Container>
  }
}