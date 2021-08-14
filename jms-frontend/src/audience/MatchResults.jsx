import React from "react";
import { Card, Col, Row } from "react-bootstrap";
import { nullIfEmpty } from "support/strings";
import { withVal } from "support/util";
import AudienceCard from "./AudienceCard";

const DERIVED_RENDER = [
  {
    key: "initiation_points",
    name: "INTIATION LINE",
    render: v => v,
  },
  {
    key: "cell_points",
    name: "POWER CELLS",
    render: v => (v.auto + v.teleop)
  },
  {
    key: "endgame_points",
    name: "ENDGAME",
    render: v => v
  },
  {
    key: "penalty_score",
    name: "OPPONENT PENALTY",
    render: v => v
  }
];

class AllianceResult extends React.PureComponent {
  render() {
    const { reverse, colour, score, winner, teams, hasrp } = this.props;
    const matchWinner = nullIfEmpty(winner?.toLowerCase());
    const win_rp = matchWinner ? ( matchWinner == colour ? 2 : 0 ) : 1;

    let top = [
      <Col>
        <Row className="teams flex-wrap">
          {
            teams.map(t => <Col>
              { t }
            </Col>)
          }
        </Row>
      </Col>,
      <Col md="auto">
        <div className="alliance-score">
          <Row>
            <Col className="total"> 
              { score.derived.total_score }
            </Col>
          </Row>
          <Row>
            <Col className="win-status">
              {
                matchWinner ? (matchWinner == colour ? "WIN" : "") : "TIE"
              }
            </Col> 
          </Row>
        </div>
      </Col>
    ];

    let bottom = [
      <Col className="breakdown" data-alliance={colour}>
        {
          DERIVED_RENDER.map(dr => <Row className="grow">
            <Col>
              { dr.name }
            </Col>
            <Col md="auto">
              { dr.render(score.derived[dr.key]) }
            </Col>
          </Row>)
        }
      </Col>,
      withVal((hasrp || undefined) && (win_rp + score.derived.total_bonus_rp), (rp) => <Col md="auto">
        <div className="alliance-rp">
          { rp } RP
        </div>
      </Col>)
    ];

    return <React.Fragment>
      <Row className="mb-3"> { reverse ? top.reverse() : top } </Row>
      <Row className="grow"> { reverse ? bottom.reverse() : bottom } </Row>
    </React.Fragment>
  }
}

export default class MatchResults extends React.PureComponent {
  render() {
    const { match, event } = this.props;

    return <AudienceCard event={event} className="audience-results">
      <Row>
        <Col className="audience-card-title" md="auto">
          { match.name }
        </Col>
        <Col className="preview-text">
          results
        </Col>
      </Row>

      <Row className="results">
        {
          ["red", "blue"].map(colour => <Col>
            <Card data-alliance={colour}>
              {
                withVal(match[`${colour}_alliance`], a => <Card.Header>
                  Alliance { a }
                </Card.Header>)
              }
              <Card.Body>
                <AllianceResult
                  reverse={colour == "blue"}
                  colour={colour}
                  score={match.score[colour]}
                  winner={match.winner}
                  teams={match[colour]} 
                  hasrp={match.type == "Qualification"}
                />
              </Card.Body>
            </Card>
          </Col>)
        }
      </Row>
    </AudienceCard>
  }
}