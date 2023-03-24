import { FieldResourceSelector, PosSelector } from "components/FieldPosSelector";
import React from "react";
import { Button, Col, Row } from "react-bootstrap";
import { Routes, Route, Link } from "react-router-dom";
import { ALLIANCES } from "support/ws-additional";
import { WebsocketComponent, withRole } from "support/ws-component";
import { Alliance, LoadedMatch, ScoreUpdateData, MatchScoreSnapshot } from "ws-schema";

// const SCORER_PAIRS: ScorerPair[] = ["AB", "CD"];

// const GOAL_HEIGHTS: GoalHeight[] = ["high", "low"];

// type ScorerPanelProps = {
//   pair: ScorerPair,
//   height: GoalHeight
// };

// type ScorerPanelState = {
//   match?: LoadedMatch,
//   score?: MatchScoreSnapshot
// };

// const GOAL_IDX = {
//   "AB": [0, 1],
//   "CD": [2, 3]
// };

// export class ScorerPanel extends WebsocketComponent<ScorerPanelProps, ScorerPanelState> {
//   readonly state: ScorerPanelState = {};

//   componentDidMount = () => this.handles = [
//     this.listen("Arena/Match/Current", "match"),
//     this.listen("Arena/Match/Score", "score"),
//   ];

//   buttonPair = (alliance: Alliance, enabled: boolean, currentVal: number, onChange: (v: number) => void) => {
//     return [
//       <Row className="grow" />,
//       <Row className="scorer-decrease">
//         <Col>
//           <Button variant="secondary" onClick={() => onChange(-1)} disabled={!enabled}> -1 </Button>
//         </Col>
//       </Row>,
//       <Row className="scorer-current">
//         <Col className={`text-${alliance}`}>
//           { currentVal }
//         </Col>
//       </Row>,
//       <Row className="scorer-increase">
//         <Col>
//           <Button variant={ alliance } onClick={() => onChange(1)} disabled={!enabled}> +1 </Button>
//         </Col>
//       </Row>
//     ]
//   }

//   scoreFlank = (goalIdx: number, match: LoadedMatch | undefined, score: MatchScoreSnapshot | undefined, update: (u: ScoreUpdateData) => void) => {
//     const arr = ALLIANCES.map(alliance => {
//       const goal = this.props.height === "high" ? "upper" : "lower";
//       const enabled = match != null && match.state !== "Waiting" && match.state !== "Fault";
//       // 5 second auto cool-off to allow for balls in the air at the end of auto
//       const teleop = match ? ( match.state === "Teleop" && match.remaining_time.secs < (match.match_meta.config.teleop_time - 5) ) : false;
//       const current_score = score ? score[alliance].live.cargo[teleop ? "teleop" : "auto"][goal] : [0, 0, 0, 0];

//       return this.buttonPair(alliance, enabled, current_score[goalIdx], n => {
//         let score_change = [0, 0, 0, 0];
//         score_change[goalIdx] = n;
//         update({ alliance, update: { Cargo: { auto: !teleop, [goal]: score_change } } });
//       });
//     });

//     return arr[0].concat(arr[1].reverse());
//   }

//   render() {
//     const { pair, height } = this.props;
//     const { match, score } = this.state;
    
//     const title = <React.Fragment>
//       <h3 className="mb-0"> { match?.match_meta?.name || "Waiting for Scorekeeper..." } </h3>
//       <i className="text-muted"> { pair }{ height[0].toUpperCase() } Scorer </i>
//     </React.Fragment>

//     const update = (u: ScoreUpdateData) => {
//       this.send({ Arena: { Match: { ScoreUpdate: u } } });
//     };

//     return <PosSelector
//       className="scorer-panel" 
//       title={title} 
//       data-pair={pair} 
//       data-height={height} 
//       img={"/img/game/hub_" + height.toLowerCase() + ".png"}
//       leftChildren={this.scoreFlank(GOAL_IDX[pair][0] - 1, match, score, update)}
//       rightChildren={this.scoreFlank(GOAL_IDX[pair][1] - 1, match, score, update)}
//     >
//       <div className="scorer-label" data-pos="left"> { pair[0] }{ height[0].toUpperCase() } </div>
//       <div className="scorer-label" data-pos="right"> { pair[1] }{ height[0].toUpperCase() } </div>
//     </PosSelector>
//   }
// };

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
    {/* {
      SCORER_PAIRS.map(pair => GOAL_HEIGHTS.map(height => (
        <Route path={`${pair}/${height}`} element={ 
          withRole({ ScorerPanel: { goals: pair, height: height } }, <ScorerPanel pair={pair} height={height} />)
        } />
      )))
    } */}
  </Routes>
}