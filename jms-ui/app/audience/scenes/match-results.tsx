import { Alliance, CommittedMatchScores, DerivedScore, EventDetails, Match, MatchScoreSnapshot, SnapshotScore, Team, TeamRanking } from "@/app/ws-schema";
import AudienceCard from "../card";
import { Card, Col, Row } from "react-bootstrap";
import { useWebsocket } from "@/app/support/ws-component";
import { useEffect, useState } from "react";
import React from "react";
import { withVal } from "@/app/support/util";
import { ALLIANCES_FLIPPED } from "@/app/support/alliances";

type DerivedRenderer<K extends keyof DerivedScore> = { key: K, name: string, render: (v: DerivedScore[K]) => number };
// Existential type hack - thanks https://stackoverflow.com/questions/65129070/defining-an-array-of-differing-generic-types-in-typescript
type SomeDerivedRenderer = <R>(cb: <T extends keyof DerivedScore>(dr: DerivedRenderer<T>) => R) => R;
const derivedRenderer = <T extends keyof DerivedScore,>(dr: DerivedRenderer<T>): SomeDerivedRenderer => cb => cb(dr);

const DERIVED_RENDER: SomeDerivedRenderer[] = [
  derivedRenderer({
    key: "leave_points",
    name: "MOBILITY POINTS",
    render: v => v,
  }),
  derivedRenderer({
    key: "notes",
    name: "NOTE POINTS",
    render: v => v.total_points
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

interface MatchResultsSceneProps {
  eventDetails: EventDetails,
  match_id: string,
  matches: Match[],
  teams: Team[]
}

export default function MatchResultsScene({ eventDetails, match_id, matches, teams }: MatchResultsSceneProps) {
  const [ score, setScores ] = useState<MatchScoreSnapshot>();
  const { call } = useWebsocket();

  const match = matches.find(m => m.id === match_id);

  useEffect(() => {
    call<"scoring/get_committed">("scoring/get_committed", { match_id })
      .then(score_record => call<"scoring/derive_score">("scoring/derive_score", { score: score_record.scores[score_record.scores.length - 1] })
                              .then(setScores));
  }, [ match_id ])

  return <AudienceCard event_name={eventDetails.event_name} className="audience-results">
    <Row>
      <Col className="audience-card-title" md="auto">
        { match?.name || match_id }
      </Col>
      <Col className="preview-text">
        results
      </Col>
    </Row>

    <Row className="results">
      {
        ALLIANCES_FLIPPED.map(alliance => <Col key={alliance as string}>
          <Card data-alliance={alliance}>
            {
              match && withVal(match[`${alliance}_alliance`], a => <Card.Header>
                Alliance { a }
              </Card.Header>)
            }
            {
              match && withVal(score, s =>
                <Card.Body>
                  <AllianceResult
                    reverse={alliance === "blue"}
                    alliance={alliance}
                    score={s[alliance]}
                    winner={s.red.derived.total_score > s.blue.derived.total_score ? "red" : s.red.derived.total_score < s.blue.derived.total_score ? "blue" : null}
                    teams={match[`${alliance}_teams`].map(t => teams.find(x => x.number === t)?.display_number || t)} 
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

type AllianceResultProps = {
  reverse?: boolean,
  alliance: Alliance,
  score: SnapshotScore,
  winner?: Alliance | null,
  teams: (number | string | null)[],
  has_rp: boolean
};

class AllianceResult extends React.PureComponent<AllianceResultProps> {
  render() {
    const { reverse, alliance, score, winner, teams, has_rp } = this.props;

    let top = [
      <Col key="top-teams">
        <Row className="teams flex-wrap">
          {
            teams.map((t, i) => <Col key={i}>
              { t }
            </Col>)
          }
        </Row>
      </Col>,
      <Col md="auto" key="top-alliance-score">
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
      <Col key="bottom-breakdown" className="breakdown" data-alliance={alliance}>
        {
          DERIVED_RENDER.map((dr, i) => <Row className="grow" key={i}>
            <Col>
              { dr(dr => dr.name) }
            </Col>
            <Col md="auto">
              { dr(dr => dr.render(score.derived[dr.key])) }
            </Col>
          </Row>)
        }
      </Col>,
      withVal((has_rp || undefined) && (score.derived.total_rp), (rp) => <Col key="bottom-rp" md="auto">
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