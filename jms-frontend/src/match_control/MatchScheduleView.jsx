import { faCheck, faExclamationTriangle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Tab } from "bootstrap";
import moment from "moment";
import { EVENT_WIZARD } from "paths";
import React from "react";
import { Button, Table, Tabs } from "react-bootstrap";

export default class MatchScheduleView extends React.Component {
  componentDidUpdate() {
    this.nextMatch = this.allMatches().find(m => !m.played);
  }

  allMatches = () => {
    if (!this.props.matches)
      return [];
    return Object.values(this.props.matches).flatMap(x => x.matches || []);
  }

  isLoaded = (match) => {
    return this.props.arena?.match?.match?.id === match.id;
  }

  // Only show this as the next match if there isn't a currently loaded match
  isNextMatch = (match) => {
    let currentMatch = this.props.arena?.match?.match;
    if (currentMatch === undefined || currentMatch === null || currentMatch.type == "Test" || currentMatch.played)
      return !match.played && (match.id === this.nextMatch?.id);
    return false;
  }

  rowClass = (match) => {
    if (this.isLoaded(match)) {
      return "loaded-match";
    } else if (match.played) {
      return "played";
    } else if (this.isNextMatch(match)) {
      return "next-match";
    } else {
      return "";
    }
  }

  howLongUntil = (match) => {
    let format = (d) => d.format("d[d] h[h] m[m]", {trim: "both"});

    let a = moment();
    let b = moment.unix(match.time);
    if (a < b)
      return "in " + format(moment.duration(b.diff(a)));
    else
      return format(moment.duration(a.diff(b))) + " ago";
  }

  renderSchedule = (matches) => {
    return <Table bordered striped size="sm">
      <thead>
        <tr>
          <th> Time </th>
          <th> Match </th>
          <th colSpan={3}> Blue </th>
          <th colSpan={3}> Red </th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody>
        {
          matches.map(match => <tr className={ this.rowClass(match) }>
            <td> 
              { moment.unix(match.time).format("dddd HH:mm:ss") }
              &nbsp;
              {
                match.played ? "" :
                <small className="text-muted">
                ({ this.howLongUntil(match) })
                </small>
              }
            </td>
            <td> 
              &nbsp; { 
                match.played ? 
                  <FontAwesomeIcon icon={faCheck} size="sm" className="text-success" /> 
                  : "" 
              } &nbsp; { match.name }
            </td>
            { match.blue.map(t => <td className="schedule-blue"> { t } </td>) }
            { match.red.map(t =>  <td className="schedule-red"> { t } </td>) }
            <td>
              {
                match.played ? "Played..." : <React.Fragment>
                  <Button 
                    onClick={() => this.props.onLoad(match)} 
                    variant={this.isNextMatch(match) ? "success" : "primary"}
                    disabled={this.props.arena?.state?.state != "Idle" || match.played || this.isLoaded(match)}
                    size="sm"
                  > 
                    LOAD
                  </Button>
                </React.Fragment>
              }
            </td>
          </tr>)
        }
      </tbody>
    </Table>
  }

  renderTab(matches) {
    return (matches?.length || 0) ? this.renderSchedule(matches) : <div className="text-center my-3">
      <h4 className="text-danger"> 
        <FontAwesomeIcon icon={faExclamationTriangle} /> 
          &nbsp; There are no matches in the schedule! &nbsp;
        <FontAwesomeIcon icon={faExclamationTriangle} /> 
      </h4>
      <p> To generate a schedule, go to the <a href={EVENT_WIZARD}>Event Wizard</a>  </p>
    </div>
  }

  render() {
    return <Tabs defaultActiveKey={ this.props.matches?.playoffs?.matches ? "playoffs" : "quals" } id="match-type-tabs">
      <Tab eventKey="quals" title="Qualifications">
        { this.renderTab(this.props.matches?.quals?.matches) }
      </Tab>
      <Tab eventKey="playoffs" title="Playoffs">
        { this.renderTab(this.props.matches?.playoffs?.matches) }
      </Tab>
    </Tabs>
  }
}