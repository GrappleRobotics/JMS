import { faCheck, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import moment from "moment";
import React from "react";
import { Button, Table } from "react-bootstrap";

export default class PlayoffGenerator extends React.Component {
  static eventKey() { return "playoff_gen"; }
  static tabName() { return "Generate Playoffs" }

  static isDisabled(d) {
    return d.alliances?.length === 0 || d.alliances?.some(x => !x.ready);
  }

  static needsAttention(d) {
    return !!!d.matches?.playoffs?.record;
  }

  renderPlayoffs = () => {
    let matches = this.props.matches?.playoffs?.matches;
    let max_teams = matches.flatMap(x => [x.blue.length, x.red.length]).reduce((a, b) => Math.max(a, b));
    return <div>
      <Button
        variant="success"
        onClick={() => this.props.ws.send("matches", "playoffs", "generate", this.props.matches.playoffs.record.data.Playoff.mode)}
      >
        Update
      </Button>
      
      <br /> <br />

      <Table bordered striped size="sm">
        <thead>
          <tr>
            <th> Time </th>
            <th> Match </th>
            <th className="schedule-blue" colSpan={max_teams}> Blue </th>
            <th className="schedule-red" colSpan={max_teams}> Red </th>
          </tr>
        </thead>
        <tbody>
          {
            matches?.map(match => <tr>
              <td> &nbsp; { moment.unix(match.time).format("ddd HH:mm:ss") } </td>
              <td> &nbsp; { match.played ? <FontAwesomeIcon icon={faCheck} size="sm" className="text-success" /> : "" } &nbsp; { match.name } </td>
              { Array.from({...match.blue, length: max_teams}).map(t => <td className="schedule-blue"> { t } </td>) }
              { Array.from({...match.red, length: max_teams}).map(t =>  <td className="schedule-red"> { t } </td>) }
            </tr>)
          }
        </tbody>
      </Table>
    </div>
  }

  renderNoPlayoffs = () => {
    // TODO: Round Robin / Other
    return <div>
      <Button
        variant="success"
        onClick={() => this.props.ws.send("matches", "playoffs", "generate", "RoundRobin")}
      >
        Generate
      </Button>
    </div> 
  }

  render() {
    return <div>
      <h4> Generate Playoff Match Schedule </h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the PLAYOFF match schedule is generated. The playoff schedule will update as matches are played.
      </p>

      <div>
        {
          this.props.matches?.playoffs?.record ? this.renderPlayoffs() : this.renderNoPlayoffs()
        }
      </div>
    </div>
  }
}