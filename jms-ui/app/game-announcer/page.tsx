"use client"

import React, { useEffect, useState } from "react"
import { AllianceStation, Match, MatchScoreSnapshot, SerialisedLoadedMatch, Team, TeamRanking } from "../ws-schema"
import { useWebsocket } from "../support/ws-component";
import { Card, Col, Row } from "react-bootstrap";
import moment from "moment";

export default function GameAnnouncerView() {
  const [ teams, setTeams ] = useState<Team[]>([]);
  const [ rankings, setRankings ] = useState<TeamRanking[]>([]);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ matchScore, setMatchScore ] = useState<MatchScoreSnapshot | null>(null);
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);
  const [ stations, setStations ] = useState<AllianceStation[]>([]);

  const { subscribe, unsubscribe } = useWebsocket();

  useEffect(() => {
    let cbs = [
      subscribe<"team/teams">("team/teams", setTeams),
      subscribe<"matches/matches">("matches/matches", setMatches),
      subscribe<"scoring/rankings">("scoring/rankings", setRankings),
      subscribe<"scoring/current">("scoring/current", setMatchScore),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
      subscribe<"arena/stations">("arena/stations", setStations)
    ];
    return () => unsubscribe(cbs);
  }, [])
  
  const match = matches.find(m => m.id === currentMatch?.match_id);
  const next_match = matches.filter(m => !m.played).sort((a, b) => moment(a.start_time).diff(moment(b.start_time)))[0];

  const renderTeam = (t: number) => {
    const team = teams.find(team => team.number === t);
    const rank = rankings.find(r => r.team === t);

    return <Row>
      <Col md={2}>
        <h4> Team { team?.display_number || t } </h4>
        <i> { rank ? `Rank: ${rankings.indexOf(rank) + 1}` : "" } </i> <br />
        <i> { rank ? `WLT: ${rank.win}-${rank.loss}-${rank.tie}` : "" } </i>
      </Col>
      <Col md={3} style={{ fontSize: '1.25em' }}>
        { team?.name || "<No Name>" }
      </Col>
      <Col md={4}>
        { team?.affiliation }
      </Col>
      <Col md={3}>
        { team?.location }
      </Col>
    </Row>
  }

  return <React.Fragment>
    <Row className="mt-3">
      <Col>
        <h3>{ match ? match.name : next_match ? <React.Fragment> <i className="text-muted">Up Next: </i> { next_match.name } </React.Fragment> : "Waiting..." }</h3>
      </Col>
      {
        currentMatch && matchScore && <Col md="auto" style={{ fontSize: '1.5em' }}>
          <strong className="text-blue">{ matchScore.blue.derived.total_score } (+{ matchScore.blue.derived.total_bonus_rp }RP)</strong> &nbsp;
          <i className="text-muted">vs</i> &nbsp;
          <strong className="text-red">{ matchScore.red.derived.total_score } (+{ matchScore.red.derived.total_bonus_rp }RP)</strong> &nbsp;
        </Col>
      }
      
    </Row>
    {
      match?.blue_alliance && match?.red_alliance && <Row>
        <Col style={{ fontSize: '1.5em' }}>
          <strong className="text-blue">Alliance { match.blue_alliance }</strong> &nbsp;
          <i className="text-muted">vs</i> &nbsp;
          <strong className="text-red">Alliance { match.red_alliance }</strong>
        </Col>
      </Row>
    }
    {
      match ? stations.map((stn, i) => <Row key={i} className="mt-2">
        <Col>
          <Card data-alliance={stn.id.alliance}>
            <Card.Body>
              { stn.team ? renderTeam(stn.team) : <h5> <i className="text-muted">Vacant</i> </h5> }
            </Card.Body>
          </Card>
        </Col>
      </Row>) : <React.Fragment>
        {
          next_match?.blue_teams.map((bt, i) => <Row key={i} className="mt-2">
            <Col>
              <Card data-alliance="blue">
                <Card.Body>
                  { bt ? renderTeam(bt) : <h5> <i className="text-muted">Vacant</i> </h5> }
                </Card.Body>
              </Card>
            </Col>
          </Row>)
        }
        {
          next_match?.red_teams.map((rt, i) => <Row key={i} className="mt-2">
            <Col>
              <Card data-alliance="red">
                <Card.Body>
                  { rt ? renderTeam(rt) : <h5> <i className="text-muted">Vacant</i> </h5> }
                </Card.Body>
              </Card>
            </Col>
          </Row>)
        }
      </React.Fragment>
    }
  </React.Fragment>
}