import { faCheck, faCircleNotch, faCog, faExclamationTriangle, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import moment from "moment";
import React from "react";
import { Button, Table } from "react-bootstrap";
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