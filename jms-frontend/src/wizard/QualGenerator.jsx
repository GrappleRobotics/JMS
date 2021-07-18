import { faCheck, faCircleNotch, faCog, faExclamationTriangle, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import moment from "moment";
import React from "react";
import { Accordion, Button, Card, Col, Form, Row, Table } from "react-bootstrap";
import { confirm } from "react-bootstrap-confirmation";

export default class QualGenerator extends React.Component {
  static eventKey() { return "qual_gen"; }
  static tabName() { return "Generate Qual Matches" }

  static isDisabled(d) {
    return ( d.teams?.length || 0 ) < 6 || (d.blocks?.filter(x => x.quals)?.length || 0) < 1;
  }

  static needsAttention(d) {
    return !!!d.quals?.matches?.length;
  }

  clearSchedule = async () => {
    let result = await confirm("Are you sure? This will clear the entire Qualification schedule", {
      title: "Clear Qualification Schedule?",
      okButtonStyle: "success"
    });

    if (result)
      this.props.ws.send("event", "quals", "delete");
  }

  renderStatsForNerds = () => {
    let gen_record = this.props.quals?.generation_record;
    return <div>
      <Row>
        <Col>
          <strong> Station Balance </strong>
          <br /> { Math.round(gen_record.station_balance * 1000) / 1000 }
          <br /> <small className="text-muted">Smaller = Better</small> 
        </Col>
        <Col>
          <strong> Team Balance </strong>
          <br /> { Math.round(gen_record.team_balance * 1000) / 1000 }
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
                gen_record.station_dist.map(r => <tr>
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
                gen_record.cooccurrence.map(r => <tr>
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
    return <div>
      <Button
        variant="danger"
        onClick={this.clearSchedule}
        disabled={this.props.quals?.locked}
      >
        { this.props.quals?.locked ? "Schedule Locked (Matches Played)" : "Clear Qualification Schedule" }
      </Button>

      <br /> <br />

      <Accordion>
        <Card>
          <Accordion.Toggle as={Card.Header} eventKey="0">
            Stats for Nerds
          </Accordion.Toggle>
          <Accordion.Collapse eventKey="0">
            <Card.Body>
              { this.renderStatsForNerds() }
            </Card.Body>
          </Accordion.Collapse>
        </Card>
      </Accordion>

      <br />

      <Table bordered striped size="sm">
        <thead>
          <tr>
            <th> Time </th>
            <th> Match </th>
            { [1,2,3].map(t => <th className="schedule-blue"> Blue {t} </th>) }
            { [1,2,3].map(t => <th className="schedule-red"> Red {t} </th>) }
          </tr>
        </thead>
        <tbody>
          {
            this.props.quals?.matches?.map(match => <tr>
              <td> &nbsp; { moment.unix(match.time).format("ddd HH:mm:ss") } </td>
              <td> &nbsp; { match.played ? <FontAwesomeIcon icon={faCheck} size="sm" className="text-success" /> : "" } &nbsp; { match.name } </td>
              { match.blue.map(t => <td className="schedule-blue"> { t } </td>) }
              { match.red.map(t =>  <td className="schedule-red"> { t } </td>) }
            </tr>)
          }
        </tbody>
      </Table>
    </div>
  }

  renderNoSchedule = () => {
    return <div>
      <Button 
        size="lg"
        variant="success" 
        onClick={ () => this.props.ws.send("event", "quals", "generate") }
        disabled={this.props.quals?.running}
      >
        <FontAwesomeIcon icon={this.props.quals?.running ? faCircleNotch : faCog} spin={this.props.quals?.running} />
        &nbsp;
        Generate Matches
      </Button>
    </div>
  }

  render() {
    return <div>
      <h4>Generate Qualification Match Schedule</h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the QUALIFICATION match schedule is generated. This will take a while. 
        <br />
        <FontAwesomeIcon icon={faExclamationTriangle} /> &nbsp;
        <strong>Teams and Schedule Blocks cannot be changed after the qualifications schedule is generated.</strong>
      </p>

      <div>
        {
          this.props.quals?.exists ? this.renderSchedule() : this.renderNoSchedule()
        }
      </div>
    </div>
  }
}