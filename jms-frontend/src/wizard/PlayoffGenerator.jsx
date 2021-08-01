import { faCheck, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import moment from "moment";
import React from "react";
import { Button, Form, Table } from "react-bootstrap";
import { confirm } from "react-bootstrap-confirmation";

export default class PlayoffGenerator extends React.Component {
  static eventKey() { return "playoff_gen"; }
  static tabName() { return "Generate Playoffs" }

  static isDisabled(d) {
    return d.alliances?.length === 0 || d.alliances?.some(x => !x.ready);
  }

  static needsAttention(d) {
    return !!!d.matches?.playoffs?.record;
  }

  constructor(props) {
    super(props);

    // For generation only
    this.state = {
      playoff_type: "Bracket"
    };
  }

  clearSchedule = async () => {
    let result = await confirm("Are you sure? This will clear the entire Playoffs schedule", {
      title: "Clear Playoffs Schedule?",
      okButtonStyle: "success"
    });

    if (result)
      this.props.ws.send("matches", "playoffs", "clear");
  }

  renderPlayoffs = () => {
    let matches = this.props.matches?.playoffs?.matches;
    let max_teams = matches.flatMap(x => [x.blue.length, x.red.length]).reduce((a, b) => Math.max(a, b));
    return <div>
      <Button
        variant="danger"
        onClick={this.clearSchedule}
        disabled={matches?.find(x => x.played)}
      >
        Clear Playoff Schedule
      </Button>
      &nbsp;
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
            <th> Match </th>
            <th className="schedule-blue" colSpan={max_teams}> Blue </th>
            <th className="schedule-red" colSpan={max_teams}> Red </th>
          </tr>
        </thead>
        <tbody>
          {
            matches?.map(match => <tr>
              <td> &nbsp; { match.played ? <FontAwesomeIcon icon={faCheck} size="sm" className="text-success" /> : "" } &nbsp; { match.name } </td>
              { Array.from({...match.blue, length: max_teams}).map(t => <td className="schedule-blue"> { t } </td>) }
              { Array.from({...match.red, length: max_teams}).map(t =>  <td className="schedule-red"> { t } </td>) }
            </tr>)
          }
        </tbody>
      </Table>
    </div>
  }

  playoffTypeSubtitle = () => {
    const n_alliances = this.props.alliances?.length;

    switch (this.state.playoff_type) {
      case "Bracket":
        const matches = n_alliances - 1;
        const next_pow = Math.pow(2, Math.ceil(Math.log2(n_alliances)));
        const n_byes = next_pow - n_alliances;
        return <span>
          { matches * 2 } matches ({ matches } sets w/ { n_byes > 0 ? `${n_byes} Byes` : "No Byes" })
        </span>;
      case "RoundRobin":
        const n = n_alliances % 2 == 0 ? n_alliances : (n_alliances + 1);
        // +2 for finals
        return <span>
          { n/2 * (n-1) + 2 } matches ({n_alliances % 2 == 0 ? "No Byes" : "w/ Bye"})
        </span>;
      default:
        return <i> Unknown... </i>;
    }
  }

  renderNoPlayoffs = () => {
    return <div>
      <Form>
        <Form.Label> Playoff Type </Form.Label>
        <Form.Control as="select" value={this.state.playoff_type} onChange={v => this.setState({ playoff_type: v.target.value })} >
          <option value="Bracket"> Elimination Bracket </option>
          <option value="RoundRobin"> Round Robin </option>
        </Form.Control>
        <Form.Text className="text-muted">
          { this.playoffTypeSubtitle() }
        </Form.Text>
      </Form>
      <br />
      <Button
        variant="success"
        onClick={() => this.props.ws.send("matches", "playoffs", "generate", this.state.playoff_type)}
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