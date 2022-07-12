import { faCheck, faExclamationTriangle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import moment from "moment";
import { EVENT_WIZARD } from "paths";
import React from "react";
import { Button, Table, Tabs, Tab } from "react-bootstrap";
import { ArenaState, LoadedMatch, SerializedMatch } from "ws-schema";

type MatchScheduleProps = {
  arenaState?: ArenaState,
  currentMatch?: LoadedMatch,
  quals: SerializedMatch[],
  playoffs: SerializedMatch[],
  nextMatch?: SerializedMatch,
  onLoadMatch: (id: string) => void
}

export default class MatchScheduleView extends React.Component<MatchScheduleProps> {
  // Only show this as the next match if there isn't a currently loaded match
  isNextMatch = (match: SerializedMatch) => {
    let currentMatch = this.props.currentMatch?.match_meta;
    if (currentMatch == null || currentMatch.match_type == "Test" || currentMatch.played)
      return !match.played && (match.id === this.props.nextMatch?.id);
    return false;
  }

  isLoaded = (match: SerializedMatch) => match.id !== undefined && this.props.currentMatch?.match_meta?.id === match.id;

  // rowClass = (match) => {
  //   if (this.isLoaded(match)) {
  //     return "loaded-match";
  //   } else if (match.played) {
  //     return "played";
  //   } else if (this.isNextMatch(match)) {
  //     return "next-match";
  //   } else {
  //     return "";
  //   }
  // }

  howLongUntil = (match: SerializedMatch) => {
    // @ts-ignore
    let format = (d: moment.Duration) => d.format("d[d] h[h] m[m]", {trim: "both"});

    if (match.start_time !== null && match.start_time !== undefined) {
      let a = moment();
      let b = moment.unix(match.start_time);
      if (a < b)
        return "in " + format(moment.duration(b.diff(a)));
      else
        return format(moment.duration(a.diff(b))) + " ago";
    } else
      return undefined;
  }

  renderSchedule = (matches: SerializedMatch[]) => {
    let { onLoadMatch, arenaState } = this.props;
    let max_teams = matches.flatMap(x => [x.blue_teams.length, x.red_teams.length]).reduce((a, b) => Math.max(a, b));

    return <Table className="match-schedule-control" bordered striped size="sm">
      <thead>
        <tr>
          <th> Time </th>
          <th> Match </th>
          <th className="schedule-row" data-alliance="blue" colSpan={max_teams}> Blue </th>
          <th className="schedule-row" data-alliance="red" colSpan={max_teams}> Red </th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody>
        {
          matches.map(match => (
            <tr 
              className={`match ${this.isLoaded(match) ? "active" : ""}`} 
              data-played={match.played || undefined} 
              data-next={this.isNextMatch(match) || undefined} 
              data-winner={ match.winner?.toLowerCase() }
            >
              {/* Time */}
              <td> 
                { match.start_time ? moment.unix(match.start_time).format("dddd HH:mm:ss") : "" }
                &nbsp;
                <small className="text-muted">
                  ({ this.howLongUntil(match) })
                </small>
              </td>
              {/* Play Status and Match Name */}
              <td> 
                &nbsp; { 
                  match.played ? 
                    <FontAwesomeIcon icon={faCheck} size="sm" className="text-good" /> 
                    : "" 
                } &nbsp; { match.name }
              </td>
              {/* Teams */}
              { Array.from({...match.blue_teams, length: max_teams}).map(t => <td className="schedule-row" data-alliance="blue"> { t } </td>) }
              { Array.from({...match.red_teams, length: max_teams}).map(t =>  <td className="schedule-row" data-alliance="red"> { t } </td>) }
              {/* Load buttons */}
              <td>
                {
                  match.played ? "Played..." 
                  : <Button 
                      className="load-match"
                      disabled={arenaState?.state !== "Idle" || match.played || this.isLoaded(match)}
                      // variant={this.isNextMatch(match) ? "success" : "primary"}
                      onClick={() => onLoadMatch(match.id || "--")} 
                      size="sm"
                    > 
                      LOAD
                    </Button>
                }
              </td>
            </tr>
          ))
        }
      </tbody>
    </Table>
  }

  renderTab(matches: SerializedMatch[]) {
    return (matches.length > 0) ? this.renderSchedule(matches) : <div className="text-center my-3">
      <h4 className="text-bad"> 
        <FontAwesomeIcon icon={faExclamationTriangle} /> 
          &nbsp; There are no matches in the schedule! &nbsp;
        <FontAwesomeIcon icon={faExclamationTriangle} /> 
      </h4>
      <p> To generate a schedule, go to the <a href={EVENT_WIZARD}>Event Wizard</a>  </p>
    </div>
  }

  render() {
    let { quals, playoffs } = this.props;
    return <Tabs defaultActiveKey={ (playoffs.length > 0) ? "playoffs" : "quals" } id="match-type-tabs">
      <Tab eventKey="quals" title="Qualifications">
        { this.renderTab(quals) }
      </Tab>
      <Tab eventKey="playoffs" title="Playoffs">
        { this.renderTab(playoffs) }
      </Tab>
    </Tabs>
  }
}