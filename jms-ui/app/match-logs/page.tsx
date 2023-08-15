"use client"
import React, { useEffect, useState } from "react";
import { Match, MatchLog, Team } from "../ws-schema";
import { useWebsocket } from "../support/ws-component";
import { Alert, Button, Col, Form, Row } from "react-bootstrap";
import { useErrors } from "../support/errors";
import MatchLogView from "./view";

export default function MatchLogsIndex() {
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ teams, setTeams ] = useState<Team[]>([]);

  const [ selectedMatch, setSelectedMatch ] = useState<Match>();
  const [ matchLog, setMatchLog ] = useState<MatchLog>();

  const { call, subscribe, unsubscribe } = useWebsocket();
  const [ error, setError ] = useState<string>();

  useEffect(() => {
    let cbs = [
      subscribe<"matches/matches">("matches/matches", setMatches),
      subscribe<"team/teams">("team/teams", setTeams),
    ];
    return () => unsubscribe(cbs);
  }, []);

  const selectMatch = (id: string) => {
    setSelectedMatch(matches.find(x => x.id === id));
    setMatchLog(undefined);
    setError(undefined);
  };

  const setMatchTeam = (match_id: string, team: number) => {
    setError(undefined);
    call<"tickets/get_match_log">("tickets/get_match_log", { match_id, team })
      .then(ml => { setMatchLog(ml); setError(undefined) })
      .catch(() => setError("No Match Log exists for Team " + team))
  };

  return <React.Fragment>
    <Row className="mt-3">
      <Col> <h3> Match Logs </h3> </Col>
    </Row>
    <Row className="mt-3">
      <Col>
        <Form.Select value={selectedMatch?.id || ""} onChange={v => selectMatch(v.target.value)}>
          <option value=""> Select Match </option>
          {
            matches.filter(m => m.played).map(m => <option key={m.id} value={m.id}>{ m.name }</option>)
          }
        </Form.Select>
      </Col>
    </Row>
    <Row className="mt-3">
      <Col>
        {
          selectedMatch && selectedMatch.blue_teams.filter(t => t !== null).map(t => <React.Fragment key={t}>
            <Button size="lg" variant="blue" key={t} onClick={() => setMatchTeam(selectedMatch.id, t!)}>
              { teams.find(team => team.number === t)?.display_number || t }
            </Button> &nbsp;
          </React.Fragment>)
        }
        {
          selectedMatch && selectedMatch.red_teams.filter(t => t !== null).map(t => <React.Fragment key={t}>
            <Button size="lg" variant="red" key={t} onClick={() => setMatchTeam(selectedMatch.id, t!)}>
              { teams.find(team => team.number === t)?.display_number || t }
            </Button> &nbsp;
          </React.Fragment>)
        }
        {
          !selectedMatch && <h4 className="text-muted">Select a Match to get started</h4>
        }
      </Col>
    </Row>
    {
      error && <Alert className="mt-3" variant="warning">{ error }</Alert>
    }
    {
      matchLog && <Row className="mt-3">
        <Col>
          <MatchLogView matchLog={matchLog} />
        </Col>
      </Row>
    }
  </React.Fragment>
}