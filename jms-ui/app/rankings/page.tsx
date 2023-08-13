"use client"
import React, { useEffect, useState } from "react";
import { useWebsocket } from "../support/ws-component";
import { useErrors } from "../support/errors";
import { Match, PlayoffMode, Team, TeamRanking } from "@/app/ws-schema";
import { Col, Row, Table } from "react-bootstrap";
import { Element, scroller } from "react-scroll";
import moment from "moment";
import _ from "lodash";
import PlayoffBracketGraph from "../components/playoff-graphs/PlayoffBracket";

const SCROLL_TIME = 20000;
const SCROLL_RESET_TIME = 2500;

export default function TeamRankings() {
  const [ rankings, setRankings ] = useState<TeamRanking[]>([]);
  const [ teams, setTeams ] = useState<Team[]>([]);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ nextMatch, setNextMatch ] = useState<Match | null>(null);
  const [ playoffMode, setPlayoffMode ] = useState<PlayoffMode>();

  const { call, subscribe, unsubscribe } = useWebsocket();

  const scrollDown = () => {
    scroller.scrollTo("bottom", {
      containerId: "rankings-container",
      duration: SCROLL_TIME,
      delay: 0,
      smooth: 'linear'
    });

    setTimeout(() => {
      scroller.scrollTo("top", {
        containerId: "rankings-container",
        delay: 0
      });
      setTimeout(() => scrollDown(), SCROLL_RESET_TIME);
    }, SCROLL_TIME + SCROLL_RESET_TIME);
  }

  const refreshPlayoffMode = () => {
    call<"matches/get_playoff_mode">("matches/get_playoff_mode", null)
      .then(setPlayoffMode)
  }

  useEffect(() => {
    let cbs = [
      subscribe<"scoring/rankings">("scoring/rankings", setRankings),
      subscribe<"matches/next">("matches/next", setNextMatch),
      subscribe<"matches/matches">("matches/matches", m => {
        setMatches(m);
        refreshPlayoffMode();
      }),
      subscribe<"team/teams">("team/teams", setTeams)
    ];
    refreshPlayoffMode();
    scrollDown();
    return () => unsubscribe(cbs);
  }, []);

  return <React.Fragment>
    <Row className="my-4">
      <Col>
        <h2> Team Standings </h2>
        {
          nextMatch ? <h4 className="text-muted"> 
            Next Match: { nextMatch.name } &nbsp; ({ moment(nextMatch.start_time).calendar() })
          </h4> : <React.Fragment />
        }
      </Col>
    </Row>
    <Row id="rankings-container">
      <Col>
        <Element name="top" />
          { 
            nextMatch?.match_type === "Playoff" ? 
              (playoffMode && <PlayoffBracketGraph matches={matches} dark_mode next_match={nextMatch} teams={teams} playoff_mode={playoffMode.mode} />)
              : <QualificationTeamRankings rankings={rankings} teams={teams} /> 
          }
        <Element name="bottom" />
      </Col>
    </Row>
  </React.Fragment>
}

function QualificationTeamRankings({ rankings, teams }: { rankings: TeamRanking[], teams: Team[] }) {
  const team_map = _.keyBy(teams, "number");
  return <Table striped bordered className="rankings">
    <thead>
      <tr>
        <th> Rank </th>
        <th> Team </th>
        <th> Played </th>
        <th> RP </th>
        <th> Auto </th>
        <th> Teleop </th>
        <th> Endgame </th>
        <th> Win-Loss-Tie </th>
      </tr>
    </thead>
    <tbody>
      {
        rankings.map((r, i) => <tr key={i} data-rank={i + 1}>
          <td> {i + 1} </td>
          <td> { team_map[r.team] ? team_map[r.team].display_number : r.team } </td>
          <td> { r.played } </td>
          <td> { r.rp } </td>
          <td> { r.auto_points } </td>
          <td> { r.teleop_points } </td>
          <td> { r.endgame_points } </td>
          <td> { r.win } - { r.loss } - { r.tie } </td>
        </tr>)
      }
    </tbody>
  </Table>
}