import moment from "moment";
import React from "react";
import { Col, Container, Row, Table } from "react-bootstrap";
import { animateScroll as scroll, scroller, Element } from "react-scroll";

const SCROLL_TIME = 20000;
const SCROLL_RESET_TIME = 2500;

export default class Rankings extends React.PureComponent {
  constructor(props) {
    super(props);

    if (this.props.scroll)
      this.scrollDown();
    
    props.ws.subscribe("event", "rankings");
    props.ws.subscribe("event", "details");
    props.ws.subscribe("matches", "next");
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
    let { rankings } = this.props;
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
    let next_match = this.props.next_match;
    return <Container className="wrapper">
      <Row className="my-4">
        <Col>
          <h2> Team Standings - { this.props.details?.event_name || "Unnamed Event" } </h2>
          {
            next_match ? <h4 className="text-muted"> 
              Next Match: { next_match.name } &nbsp; ({ moment.unix(next_match.time).calendar() })
            </h4> : <React.Fragment />
          }
        </Col>
      </Row>
      <Row id="rankings-container" className="app-viewport">
        <Col>
            <Element name="top" />
            {
              this.props.rankings?.length ? this.renderRankings() : <h4> No Rankings Available - waiting for matches to begin... </h4>
            }
            <Element name="bottom" />
        </Col>
      </Row>
    </Container>
  }

}