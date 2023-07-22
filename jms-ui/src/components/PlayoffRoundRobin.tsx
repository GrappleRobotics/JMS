import React from "react";
import { Col, Row } from "react-bootstrap";
import { SerialisedMatchGeneration, SerializedMatch } from "ws-schema";
import { ALLIANCES } from "support/ws-additional";
import _ from "lodash";

export type PlayoffRoundRobinProps = {
  gen_record: SerialisedMatchGeneration,
  next?: SerializedMatch,
  dark_mode?: boolean
}

export default class PlayoffRoundRobin extends React.PureComponent<PlayoffRoundRobinProps> {
  render() {
    const { gen_record, next, dark_mode } = this.props;
    let alliance_records: ({ alliance: number, teams: number[], win: number, loss: number, tie: number } | undefined)[] = [];

    gen_record.matches.filter(m => m.played).forEach(m => {
      let red_a = m.red_alliance!;
      let blue_a = m.blue_alliance!;

      alliance_records[red_a] ||= { alliance: red_a, teams: m.red_teams.filter(t => t != null) as number[], win: 0, loss: 0, tie: 0 };
      alliance_records[blue_a] ||= { alliance: blue_a, teams: m.blue_teams.filter(t => t != null) as number[], win: 0, loss: 0, tie: 0 };

      if (m.winner == null) {
        // Tie
        alliance_records[red_a]!.tie += 1;
        alliance_records[blue_a]!.tie += 1;
      } else {
        let winner = m.winner === "blue" ? blue_a : red_a;
        let loser = m.winner === "blue" ? red_a : blue_a;

        alliance_records[winner]!.win += 1;
        alliance_records[loser]!.loss += 1;
      }
    });

    let sorted_records = alliance_records.filter(t => t != null);
    sorted_records.sort((a, b) => b!.win * 2 + b!.tie - a!.win * 2 - a!.tie);

    return <React.Fragment>
      <Col className="round-robin-matches" data-dark-mode={dark_mode}>
        <Row>
          <Col className="round-robin-title"> Round-Robin Semifinals </Col>
        </Row>
        {
          gen_record.matches.filter(m => !m.played).map(m => <Row key={m.id || ""} className="round-robin-match" data-next={next?.id === m.id}>
            <Col className="round-robin-match-name"> { m.short_name } </Col>
            {
              ALLIANCES.map(alliance => <Col className="round-robin-alliance" data-alliance={alliance}>
                <Row>
                  <Col md="auto" className="round-robin-alliance-num">
                    { m[`${alliance}_alliance`] }
                  </Col>
                  <Col>
                    <Row>
                      {
                        m[`${alliance}_teams`].filter(t => t != null).map(t => <Col>
                          {t || ""}
                        </Col>)
                      }
                    </Row>
                  </Col>
                </Row>
              </Col>)
            }
          </Row>)
        }
      </Col>
      <Col className="round-robin-standings" data-has-leaderboard={sorted_records.length > 0} data-dark-mode={dark_mode}>
        <Row>
          <Col className="round-robin-title"> Leaderboard </Col>
        </Row>
        <Row className="round-robin-standings-title">
          <Col md={1}> # </Col>
          <Col md={8}> Alliance </Col>
          <Col md={3}> W-L-T </Col>
        </Row>
        {
          sorted_records.map((r, i) => <Row className="round-robin-standings-entry">
            <Col md={1}> <strong>{ i + 1 }</strong> </Col>
            <Col md={8}> { r?.teams.join("-") } </Col>
            <Col md={3}> { r?.win }-{ r?.loss }-{ r?.tie } </Col>
          </Row>)
        }
      </Col>
    </React.Fragment>
  }
};