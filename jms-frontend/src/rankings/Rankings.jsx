import moment from "moment";
import React from "react";
import { Container, Table } from "react-bootstrap";

export default class Rankings extends React.PureComponent {

  allMatches = () => {
    if (!this.props.matches)
      return [];
    return Object.values(this.props.matches).flatMap(x => x.matches || []);
  }

  renderRankings = () => {
    let { rankings } = this.props;
    return <Table bordered striped className="rankings">
      <thead>
        <tr>
          <th> Rank </th>
          <th> Team </th>
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
    let next_match = this.allMatches().find(m => !m.played);
    return <Container className="my-4">
      <h2> Team Standings - { this.props.details?.event_name || "Unnamed Event" } </h2>
      {
        next_match ? <h4 className="text-muted"> 
          Next Match: { next_match.name } &nbsp; ({ moment.unix(next_match.time).calendar() })
        </h4> : <React.Fragment />
      }
      
      <br />
      {
        this.props.rankings?.length ? this.renderRankings() : <h4> No Rankings Available - waiting for matches to begin... </h4>
      }
    </Container>
  }

}