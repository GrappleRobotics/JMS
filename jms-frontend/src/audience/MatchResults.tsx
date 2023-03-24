import React from "react";
import { Card, Col, Row } from "react-bootstrap";
import { withVal } from "support/util";
import { ALLIANCES } from "support/ws-additional";
import { Alliance, DerivedScore, SerializedMatch, SnapshotScore } from "ws-schema";
import AudienceCard from "./AudienceCard";
import BaseAudienceScene from "./BaseAudienceScene";

type DerivedRenderer<K extends keyof DerivedScore> = { key: K, name: string, render: (v: DerivedScore[K]) => number };
// Existential type hack - thanks https://stackoverflow.com/questions/65129070/defining-an-array-of-differing-generic-types-in-typescript
type SomeDerivedRenderer = <R>(cb: <T extends keyof DerivedScore>(dr: DerivedRenderer<T>) => R) => R;
const derivedRenderer = <T extends keyof DerivedScore,>(dr: DerivedRenderer<T>): SomeDerivedRenderer => cb => cb(dr);

const DERIVED_RENDER: SomeDerivedRenderer[] = [
  derivedRenderer({
    key: "mobility_points",
    name: "MOBILITY POINTS",
    render: v => v,
  }),
  derivedRenderer({
    key: "auto_docked_points",
    name: "AUTO DOCK POINTS",
    render: v => v,
  }),
  derivedRenderer({
    key: "community_points",
    name: "MOBILITY POINTS",
    render: v => (v.auto + v.teleop)
  }),
  derivedRenderer({
    key: "link_points",
    name: "LINK POINTS",
    render: v => v
  }),
  derivedRenderer({
    key: "endgame_points",
    name: "ENDGAME",
    render: v => v
  }),
  derivedRenderer({
    key: "penalty_score",
    name: "OPPONENT PENALTY",
    render: v => v
  })
];

type AllianceResultProps = {
  reverse?: boolean,
  alliance: Alliance,
  score: SnapshotScore,
  winner?: Alliance | null,
  teams: (number | null)[],
  has_rp: boolean
};

class AllianceResult extends React.PureComponent<AllianceResultProps> {
  render() {
    const { reverse, alliance, score, winner, teams, has_rp } = this.props;

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
                winner ? (winner === alliance ? "WIN" : "") : "TIE"
              }
            </Col> 
          </Row>
        </div>
      </Col>
    ];

    let bottom = [
      <Col className="breakdown" data-alliance={alliance}>
        {
          DERIVED_RENDER.map(dr => <Row className="grow">
            <Col>
              { dr(dr => dr.name) }
            </Col>
            <Col md="auto">
              { dr(dr => dr.render(score.derived[dr.key])) }
            </Col>
          </Row>)
        }
      </Col>,
      withVal((has_rp || undefined) && (score.derived.total_rp), (rp) => <Col md="auto">
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

type AudienceSceneMatchResultsProps = {
  match: SerializedMatch
};

export default class AudienceSceneMatchResults extends BaseAudienceScene<AudienceSceneMatchResultsProps> {
  show = (props: AudienceSceneMatchResultsProps) => {
    const { match } = props;

    return <AudienceCard event_name={this.props.details.event_name} className="audience-results">
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
          ALLIANCES.map(alliance => <Col>
            <Card data-alliance={alliance}>
              {
                withVal(match[`${alliance}_alliance`], a => <Card.Header>
                  Alliance { a }
                </Card.Header>)
              }
              {
                withVal(match.full_score, score =>
                  <Card.Body>
                    <AllianceResult
                      reverse={alliance === "blue"}
                      alliance={alliance}
                      score={score[alliance]}
                      winner={match.winner}
                      teams={match[`${alliance}_teams`]} 
                      has_rp={match.match_type === "Qualification"}
                    />
                  </Card.Body>
                )
              }
            </Card>
          </Col>)
        }
      </Row>
    </AudienceCard>
  }
}