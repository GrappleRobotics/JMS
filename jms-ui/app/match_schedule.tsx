import "./match_schedule.scss";
import { Button, Col, Row, Table } from "react-bootstrap";
import { Match, SerialisedLoadedMatch, Team } from "./ws-schema";
import React from "react";
import moment from "moment";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCheck, faTrash } from "@fortawesome/free-solid-svg-icons";
import { withConfirm } from "./components/Confirm";
import { useWebsocket } from "./support/ws-component";
import { useErrors } from "./support/errors";

interface MatchScheduleProps {
  matches: Match[],
  currentMatch?: SerialisedLoadedMatch,
  canLoad?: boolean,
  isLoadDisabled?: boolean,
  canDelete?: boolean,
  filter?: (m: Match) => boolean,
  teams?: Team[]
}

export default function MatchSchedule({ matches, currentMatch, canLoad, isLoadDisabled, canDelete, filter, teams }: MatchScheduleProps) {
  const { call } = useWebsocket();
  const { addError } = useErrors();

  return <React.Fragment>
    {
      canLoad && <Row className="mb-2">
        <Col>
          <h4><i>{ currentMatch && currentMatch.match_id === "test" ? "Test Match" : matches.find(m => m.id === currentMatch?.match_id)?.name || currentMatch?.match_id }</i></h4>
        </Col>
        <Col md="auto">
          <Button variant="primary" disabled={isLoadDisabled} onClick={() => call<"arena/load_test_match">("arena/load_test_match", null).catch(addError)}>
            LOAD TEST MATCH
          </Button> &nbsp; 
          <Button variant="danger" disabled={isLoadDisabled} onClick={() => call<"arena/unload_match">("arena/unload_match", null).catch(addError)}>
            UNLOAD MATCH
          </Button>
        </Col>
      </Row>
    }
    <Row>
      <Col>
        <Table className="match-schedule" bordered striped size="sm">
          <thead>
            <tr className="schedule-row">
              <th> Time </th>
              <th> Match </th>
              <th className="schedule-row" data-alliance="blue" colSpan={4}> Blue </th>
              <th className="schedule-row" data-alliance="red" colSpan={4}> Red </th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {
              matches.filter(filter || ((m: Match) => true)).map(match => <tr className="schedule-row" data-played={match.played}>
                <td> { moment(match.start_time).format("ddd HH:mm:ss") } </td>
                <td> { match.name } { match.played && <span className="text-success">&nbsp;<FontAwesomeIcon icon={faCheck} /></span> } </td>
                <td data-alliance="blue"> <strong>{ match.blue_alliance ? `#${match.blue_alliance}` : "" }</strong> </td>
                {
                  match.blue_teams.map(t => <td data-alliance="blue">{ teams?.find(x => x.number === t)?.display_number || t }</td>)
                }
                <td data-alliance="red"> <strong>{ match.red_alliance ? `#${match.red_alliance}` : "" }</strong> </td>
                {
                  match.red_teams.map(t => <td data-alliance="red">{ teams?.find(x => x.number === t)?.display_number || t }</td>)
                }
                <td>
                  { canDelete && <Button variant="danger" size="sm" disabled={match.played} onClick={() => withConfirm(() => call<"matches/delete">("matches/delete", { match_id: match.id }).catch(addError))}>
                    <FontAwesomeIcon icon={faTrash} />
                  </Button>}
                  { canLoad && <Button variant="primary" size="sm" disabled={isLoadDisabled || !match.ready} onClick={() => call<"arena/load_match">("arena/load_match", { match_id: match.id }).catch(addError)}>
                    LOAD
                  </Button> }
                </td>
              </tr>)
            }
          </tbody>
        </Table>
      </Col>
    </Row>
  </React.Fragment>
}