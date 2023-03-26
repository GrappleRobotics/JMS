import { FieldResourceSelector, PosSelector } from "components/FieldPosSelector";
import React from "react";
import { Button, Col, Row } from "react-bootstrap";
import { Routes, Route, Link } from "react-router-dom";
import { withVal } from "support/util";
import { ALLIANCES } from "support/ws-additional";
import { WebsocketComponent, withRole } from "support/ws-component";
import { Alliance, LoadedMatch, ScoreUpdateData, MatchScoreSnapshot, GamepieceType } from "ws-schema";

type ScorerPanelProps = {
  alliance: Alliance
};

type ScorerPanelState = {
  match?: LoadedMatch,
  score?: MatchScoreSnapshot,
  autoFinalised: boolean
};

const ALLOWED_GAMEPIECE_MAP = [
  Array(9).fill(["None", "Cube", "Cone"]),
  [ ["None", "Cone"], ["None", "Cube"], ["None", "Cone"], ["None", "Cone"], ["None", "Cube"], ["None", "Cone"], ["Cone"], ["None", "Cube"], ["None", "Cone"] ],
  [ ["None", "Cone"], ["None", "Cube"], ["None", "Cone"], ["None", "Cone"], ["None", "Cube"], ["None", "Cone"], ["Cone"], ["None", "Cube"], ["None", "Cone"] ],
];

export class ScorerPanel extends WebsocketComponent<ScorerPanelProps, ScorerPanelState> {
  readonly state: ScorerPanelState = { autoFinalised: false };

  componentDidMount = () => this.handles = [
    this.listen("Arena/Match/Current", "match"),
    this.listen("Arena/Match/Score", "score"),
  ];

  render() {
    const { alliance } = this.props;
    const { match, score, autoFinalised } = this.state;
    
    let my_score = score?.[alliance];

    const title = <React.Fragment>
      <h3 className="mb-0"> { match?.match_meta?.name || "Waiting for Scorekeeper..." } </h3>
      <i className="text-muted"> { alliance.toUpperCase() } Scorer </i>
    </React.Fragment>

    return <div className="scorer-panel">
      <Row className="mb-3">
        <Col>
          { title }
        </Col>
        <Col md="auto">
          {
            withVal(match, m => (
              <Button
                className="scorer-auto-finalise"
                data-finalised={autoFinalised}
                disabled={autoFinalised || (m.state == "Auto" || m.state == "Waiting" || m.state == "Warmup")}
                onClick={() => this.setState({ autoFinalised: !autoFinalised })}
              > 
                { autoFinalised ? <React.Fragment>AUTO FINALISED</React.Fragment> : <React.Fragment>FINALISE AUTO</React.Fragment> }
              </Button>
            ))
          }
        </Col>
      </Row>
      {
        withVal(my_score, s => {
          // let community = s.live.community[autoFinalised ? "teleop" : "auto"];
          let auto = s.live.community.auto;
          let teleop = s.live.community.teleop;

          let merged_community: GamepieceType[][] = [ Array(9).fill("None"), Array(9).fill("None"), Array(9).fill("None") ];
          for (let i = 0; i < 3; i++) {
            for (let j = 0; j < 9; j++) {
              if (auto[i][j] != "None") merged_community[i][j] = auto[i][j];
              if (teleop[i][j] != "None") merged_community[i][j] = teleop[i][j];
            }
          }

          return <div>
            <Row className="scorer-community">
              <Col>
                {
                  merged_community.map((row, i) => <Row className="scorer-community-row">
                    {
                      row.map((gamepiece, j) => (
                        <Col
                          className="scorer-community-col"
                          data-alliance={alliance}
                          data-column={j}
                          data-has-auto={auto[i][j] != "None"}
                          onClick={() => {
                            let allowed: GamepieceType[] = ALLOWED_GAMEPIECE_MAP[i][j];
                            let current_idx = allowed.findIndex(g => g == gamepiece);
                            let next = (current_idx + 1) % allowed.length;
                            this.send({ Arena: { Match: { ScoreUpdate: { alliance: alliance, update: { Community: { auto: !autoFinalised, row: i, col: j, gamepiece: allowed[next] } } } } } });
                          }}
                        >
                          <div className="scorer-gamepiece" data-gamepiece={gamepiece} />
                        </Col>
                      ))
                    }
                  </Row>).reverse()
                }
              </Col>
            </Row>
          </div>
        })
      }
    </div>
  }
}

class ScorerSelector extends React.PureComponent {
  render() {
    return <Col className="col-full">
      <FieldResourceSelector
        title="Select Scorer"
        options={[
          { ScorerPanel: { alliance: "blue" } },
          { ScorerPanel: { alliance: "red" } },
        ]}
        labels={[ "BLUE", "RED" ]}
        wrap={(r, child) => <Link to={`${r.ScorerPanel.alliance}`}>{child}</Link>}
      />
    </Col>
  }
}

export function ScoringRouter() {
  return <Routes>
    <Route path="/" element={ <ScorerSelector /> } />
    {
      ALLIANCES.map(alliance => <Route path={`${alliance}`} element={ withRole({ ScorerPanel: { alliance: alliance } }, <ScorerPanel alliance={alliance} />) } />)
    }
  </Routes>
}