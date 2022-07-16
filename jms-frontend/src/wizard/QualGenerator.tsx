import { faCheck, faCircleNotch, faCog, faExclamationTriangle, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import confirmBool from "components/elements/Confirm";
import _ from "lodash";
import moment from "moment";
import { Accordion, Button, Col, Form, Row, Table } from "react-bootstrap";
import { WebsocketComponent } from "support/ws-component";
import { MatchGenerationRecordData, ScheduleBlock, SerialisedMatchGeneration, Team } from "ws-schema";
import { EventWizardPageContent } from "./EventWizard";

type QualGeneratorState = {
  gen?: SerialisedMatchGeneration,
  team_anneal_steps: number,
  station_anneal_steps: number,
  teams: Team[],
  schedule: ScheduleBlock[]
};

type QualGenRecord = Extract<MatchGenerationRecordData, { Qualification: any }>;

export default class QualGenerator extends WebsocketComponent<{}, QualGeneratorState> {
  
  readonly state: QualGeneratorState = {
    team_anneal_steps: 100_000,
    station_anneal_steps: 50_000,
    teams: [],
    schedule: []
  };

  componentDidMount = () => this.handles = [
    this.listen("Event/Team/CurrentAll", "teams"),
    this.listen("Match/Quals/Generation", "gen"),
    this.listen("Event/Schedule/CurrentBlocks", "schedule")
  ]

  clearSchedule = async () => {
    let result = await confirmBool("Are you sure? This will clear the entire Qualification schedule", {
      title: "Clear Qualification Schedule?",
      okVariant: "danger",
      okText: "Clear Quals Matches"
    });

    if (result)
      this.send({ Match: { Quals: "Clear" } });
  }

  renderStatsForNerds = (record: QualGenRecord["Qualification"]) => {
    return <div>
      <Row>
        <Col>
          <strong> Station Balance </strong>
          <br /> { Math.round(record.station_balance * 1000) / 1000 }
          <br /> <small className="text-muted">Smaller = Better</small> 
        </Col>
        <Col>
          <strong> Team Balance </strong>
          <br /> { Math.round(record.team_balance * 1000) / 1000 }
          <br /> <small className="text-muted">Smaller = Better</small> 
        </Col>
      </Row>
     <Row className="mt-3">
        <Col md={5}>
          <strong> Station Distribution </strong>
          <br />
          <Table size="sm">
            <tbody>
              {
                record.station_dist.map(r => <tr>
                  { r.map(c => <td> {c} </td>) }
                </tr>)
              }
            </tbody>
          </Table>
        </Col>
        <Col md={7}>
          <strong> Team Cooccurrence </strong>
          <br />
          <Table size="sm">
            <tbody>
              {
                record.cooccurrence.map(r => <tr>
                  { r.map(c => <td> {c} </td>) }
                </tr>)
              }
            </tbody>
          </Table>
        </Col>
      </Row>
    </div>
  }

  renderSchedule = () => {
    const { gen, teams } = this.state;
    const matches = gen?.matches || [];
    const record = (gen?.record?.data! as QualGenRecord)["Qualification"];

    const mean_gap = _.mean(teams.flatMap(t => {
      const matchIdxs = _.filter( _.range(matches.length), (i) => _.find(matches[i].blue_teams, u => u === t.id) != null || _.find(matches[i].red_teams, u => u === t.id) != null);
      const diff = _.zip(matchIdxs.slice(1), matchIdxs.slice(0, matchIdxs.length - 1)).map(v => (v!)[0]! - (v!)[1]!);
      return diff;
    }));

    return <div>
      <Button
        variant="danger"
        onClick={this.clearSchedule}
        disabled={matches.find(x => x.played) != null}
      >
        Clear Qualification Schedule
      </Button>

      <br /> <br />

      <Accordion>
        {/* <Card>
          <Accordion.Toggle as={Card.Header} eventKey="0">
            Stats for Nerds
          </Accordion.Toggle>
          <Accordion.Collapse eventKey="0">
            <Card.Body>
              { (record && "Qualification" in record) ? this.renderStatsForNerds(record.Qualification) : undefined }
            </Card.Body>
          </Accordion.Collapse>
        </Card> */}
        <Accordion.Item eventKey="0">
          <Accordion.Header> Stats for Nerds </Accordion.Header>
          <Accordion.Body>
            { this.renderStatsForNerds(record) }
          </Accordion.Body>
        </Accordion.Item>
      </Accordion>

      <br />

      <span> Schedule Generated in { moment.duration(record.gen_time, 'milliseconds').asSeconds().toFixed(2) }s </span>
      <br />
      <span> Mean Gap: { mean_gap.toFixed(2) } matches </span>

      <br />
      <br />

      <Table bordered striped size="sm">
        <thead>
          <tr>
            <th> Time </th>
            <th> Match </th>
            { [1,2,3].map(t => <th className="schedule-row" data-alliance="blue"> Blue {t} </th>) }
            { [1,2,3].map(t => <th className="schedule-row" data-alliance="red"> Red {t} </th>) }
          </tr>
        </thead>
        <tbody>
          {
            matches.map(match => <tr>
              <td> &nbsp; { match.start_time ? moment.unix(match.start_time).format("ddd HH:mm:ss") : "Unknown" } </td>
              <td> &nbsp; { match.played ? <FontAwesomeIcon icon={faCheck} size="sm" className="text-success" /> : "" } &nbsp; { match.name } </td>
              { match.blue_teams.map(t => <td className="schedule-row" data-alliance="blue"> { t } </td>) }
              { match.red_teams.map(t =>  <td className="schedule-row" data-alliance="red"> { t } </td>) }
            </tr>)
          }
        </tbody>
      </Table>
    </div>
  }

  renderNoSchedule = () => {
    const { team_anneal_steps, station_anneal_steps } = this.state;
    const running = this.state.gen?.running || false;

    return <div>
      <Accordion>
        <Accordion.Item eventKey="0">
          <Accordion.Header>Advanced Configuration</Accordion.Header>
          <Accordion.Body>
            <Row>
              <Col>
                <Form.Label> Team Balance Annealer Steps </Form.Label>
                <Form.Control
                  type="number"
                  value={this.state.team_anneal_steps}
                  onChange={e => this.setState({ team_anneal_steps: parseInt(e.target.value) })}
                />
                <Form.Text> { this.state.team_anneal_steps.toLocaleString() } </Form.Text>
              </Col>
              <Col>
                <Form.Label> Station Balance Annealer Steps </Form.Label>
                <Form.Control
                  type="number"
                  value={this.state.station_anneal_steps}
                  onChange={e => this.setState({ station_anneal_steps: parseInt(e.target.value) })}
                />
                <Form.Text> { this.state.station_anneal_steps.toLocaleString() } </Form.Text>
              </Col>
            </Row>
          </Accordion.Body>
        </Accordion.Item>
      </Accordion>

      <br />
      <Button 
        size="lg"
        variant="success" 
        onClick={ () => this.send({ Match: { Quals: { Generate: { team_anneal_steps, station_anneal_steps } } } }) }
        disabled={running}
      >
        <FontAwesomeIcon icon={running ? faCircleNotch : faCog} spin={running} />
        &nbsp;
        Generate Matches
      </Button>
    </div>
  }

  render() {
    const has_generated = this.state.gen?.record?.data;
    const prereq = this.state.teams.length > 6 && _.some(this.state.schedule, s => s.block_type === "Qualification");
    
    return <EventWizardPageContent tabLabel="Generate Qualification Matches" attention={!has_generated} disabled={!prereq}>
      <h4>Generate Qualification Match Schedule</h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the QUALIFICATION match schedule is generated. This will take a while. 
        <br />
        <FontAwesomeIcon icon={faExclamationTriangle} /> &nbsp;
        <strong>Teams and Schedule cannot be changed after the qualifications schedule is generated.</strong>
      </p>

      <div>
        {
          has_generated ? this.renderSchedule() : this.renderNoSchedule()
        }
      </div>
    </EventWizardPageContent>
  }
}