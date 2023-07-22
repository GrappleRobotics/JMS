import PlayoffBracketGraph, { as_playoffs } from "components/PlayoffBracket";
import PlayoffDoubleBracketGraph from "components/PlayoffDoubleBracket";
import PlayoffRoundRobin from "components/PlayoffRoundRobin";
import moment from "moment";
import React from "react";
import { Col, Container, Row, Table } from "react-bootstrap";
import { Element, scroller } from "react-scroll";
import { WebsocketComponent } from "support/ws-component";
import { EventDetails, SerialisedMatchGeneration, SerializedMatch, TeamRanking } from "ws-schema";

const SCROLL_TIME = 20000;
const SCROLL_RESET_TIME = 2500;

type RankingsProps = {
  scroll: boolean
};

type RankingsState = {
  rankings: TeamRanking[],
  event_details?: EventDetails,
  playoffs?: SerialisedMatchGeneration,
  next_match?: SerializedMatch
}

export default class Rankings extends WebsocketComponent<RankingsProps, RankingsState> { 
  static defaultProps = {
    scroll: true
  };
  
  readonly state: RankingsState = {
    rankings: []
  };

  componentDidMount = () => {
    this.handles = [
      this.listen("Event/Ranking/CurrentAll", "rankings"),
      this.listen("Event/Details/Current", "event_details"),
      this.listen("Match/Playoffs/Generation", "playoffs"),
      this.listen("Match/Next", "next_match"),
    ];
    if (this.props.scroll)
      this.scrollDown();
  }

  scrollDown = () => {
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
      setTimeout(() => this.scrollDown(), SCROLL_RESET_TIME);
    }, SCROLL_TIME + SCROLL_RESET_TIME);
  }

  renderRankings = () => {
    const { rankings } = this.state;
    return <Table bordered striped className="rankings">
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
          rankings.map((r, i) => <tr data-rank={i + 1}>
            <td> {i + 1} </td>
            <td> { r.team } </td>
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

  render() {
    const { event_details, rankings, playoffs, next_match } = this.state;
    const playoff_data = as_playoffs(playoffs?.record?.data);

    return <Container className="wrapper">
      <Row className="my-4">
        <Col>
          <h2> Team Standings - { event_details?.event_name || "Unnamed Event" } </h2>
          {
            next_match ? <h4 className="text-muted"> 
              Next Match: { next_match.name } &nbsp; ({ next_match.start_time ? moment.unix(next_match.start_time).calendar() : "soon" })
            </h4> : <React.Fragment />
          }
        </Col>
      </Row>
      <Row id="rankings-container" className="app-viewport">
        <Col className="col-full">
            <Element name="top" />
            {
              playoff_data ? (
                playoff_data?.mode === "Bracket"
                  ? <PlayoffBracketGraph dark_mode gen_record={playoffs!} next={next_match} />
                  : playoff_data?.mode === "DoubleBracket" ? <PlayoffDoubleBracketGraph dark_mode gen_record={playoffs!} next={next_match} />
                  : <Row className="grow" style={{ fontSize: "2.5vw", maxHeight: "80vh" }}>
                      <PlayoffRoundRobin dark_mode gen_record={playoffs!} next={next_match} />
                    </Row>
              ) :
              rankings.length > 0 ?
                this.renderRankings()
                : <h4> No Rankings Available - waiting for matches to begin... </h4>
            }
            <Element name="bottom" />
        </Col>
      </Row>
    </Container>
  }

}