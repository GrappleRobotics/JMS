import React from "react";
import { Col, Row } from "react-bootstrap";
import AudienceCard from "./AudienceCard";

export default class AllianceSelection extends React.PureComponent {
  render() {
    const { event } = this.props;
    const { alliances, rankings } = event;

    const chosen = alliances.flatMap(a => a.teams).filter(x => !!x);
    const remaining = rankings.filter(r => !chosen.includes(r.team)).map((r, i) => (
      { id: r.team.toString(), rank: i + 1 }
    ));

    return <AudienceCard event={event} className="audience-alliance-selection">
      <Row>
        <Col md={7} className="audience-card-title"> Alliance Selection </Col>
        <Col className="audience-card-title text-right"> Remaining Teams </Col>
      </Row>
      <Row className="grow">
        <Col className="alliance-table" md={8}>
          <Row>
            <Col md={1}> # </Col>
            <Col> Captain </Col>
            <Col> Pick 1 </Col>
            <Col> Pick 2 </Col>
            <Col> Pick 3 </Col>
          </Row>
          {
            alliances.map(alliance => <Row>
              <Col md={1}> { alliance.id } </Col>
              {
                [0, 1, 2, 3].map(i => <Col>
                  { alliance.teams[i] }
                </Col>)
              }
            </Row>)
          }
        </Col>
        <Col className="alliance-remaining">
          <Row className="flex-wrap">
            {
              remaining.map(r => <Col md="auto">
                <span className="rank">{ r.rank }</span>: { r.id }
              </Col>)
            }
          </Row>
        </Col>
      </Row>
    </AudienceCard>
  }
}