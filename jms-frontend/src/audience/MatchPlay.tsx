import ProgressBar from "react-bootstrap/ProgressBar";
import React from "react";
import { Col, Row } from "react-bootstrap";
import { withVal } from "support/util";
import { Alliance, AllianceStation, Duration, LoadedMatch, MatchConfig, MatchPlayState, SnapshotScore } from "ws-schema";
import BaseAudienceScene from "./BaseAudienceScene";

type MatchProgressBarProps = {
  config: MatchConfig,
  remaining: Duration,
  state: MatchPlayState,
  endgame: boolean
};

class MatchProgressBar extends React.PureComponent<MatchProgressBarProps> {
  render() {
    const { config, remaining, state, endgame } = this.props;

    let bars = [
      {
        name: "AUTONOMOUS",
        max: config.auto_time.secs,
        state: "Auto",
        complete: ["Pause", "Teleop", "Cooldown", "Complete"]
      },
      {
        name: "TELEOP",
        max: config.teleop_time.secs,
        state: "Teleop",
        complete: ["Cooldown", "Complete"]
      }
    ];

    let total = bars.reduce((a, b) => a + b.max, 0);

    return <React.Fragment>
      {
        bars.map(bar =>
          <ProgressBar
            className="match-progress"
            data-state={ bar.state }
            data-active={ bar.state === state }
            data-fault={ state === "Fault" }
            data-endgame={ endgame }
            style={{
              width: `${bar.max / total * 100}vw`
            }}

            animated={ bar.state === state || state === "Fault" }
            max={ bar.max }
            now={ 
              bar.state === state ? 
              (bar.max - remaining.secs) : 
              bar.complete.find(s => s === state) ? bar.max :
              state === "Fault" ? bar.max : 0 }
          />
        )
      }
    </React.Fragment>
  }
}

type AllianceScoreProps = {
  reverse?: boolean,
  has_rp: boolean,
  alliance: Alliance,
  score: SnapshotScore,
  stations: AllianceStation[],
  img?: string
}

class AllianceScore extends React.PureComponent<AllianceScoreProps> {
  render() {
    const { reverse, alliance, score, stations, img, has_rp } = this.props;

    const els = [
      <Col className="score-image">
        {
          withVal(img, () => <img src={`/img/${img}`} />)
        }
      </Col>,
      <Col className="alliance-teams" data-alliance={alliance}>
        {
          stations.map(s => 
            <Row 
              className="alliance-team"
              data-bypass={s.bypass}
              data-estop={s.estop || s.astop}
              data-alive={s.ds_report?.robot_ping}
            >
              <Col> { s.team || "\u00A0" } </Col>
            </Row>
          )
        }
      </Col>,
      <Col className="total-score" data-alliance={alliance}>
        { score.derived.total_score }
        {
          withVal((has_rp && score.derived.total_bonus_rp) || undefined, bonus => <span className="total-score-bonus-rp">
            +{ score.derived.total_bonus_rp } RP
          </span>)
        }
      </Col>
    ];

    return reverse ? els.reverse() : els;
  }
}

type AudienceSceneMatchPlayState = {
  stations: AllianceStation[],
  match?: LoadedMatch
};

export default class AudienceSceneMatchPlay extends BaseAudienceScene<{}, AudienceSceneMatchPlayState> {
  readonly state: AudienceSceneMatchPlayState = { stations: [] };
  
  componentDidMount = () => this.handles = [
    this.listen("Arena/Alliance/CurrentStations", "stations"),
    this.listen("Arena/Match/Current", "match")
  ];

  show = () => {
    // const { arena, event } = this.props;
    // const { match } = arena;
    const { match, stations } = this.state;

    if (match == null)
      return <div className="audience-field" />
    else {
      const has_rp = match.match_meta.match_type === "Qualification";

      return <div className="audience-play">
        <div className="score-block">
          <Row className="m-0 progress-row">
            <MatchProgressBar
              config={match.config}
              remaining={match.remaining_time}
              state={match.state}
              endgame={match.endgame}
            />
            <span className="progress-overlay">
              <Col>
                { match.match_meta.name }
              </Col>
              <Col md={2}>
                { match.state } &nbsp;
                { 
                  match.state === "Waiting" 
                    || match.state === "Complete"
                    || `${match.remaining_time.secs}s`
                }
              </Col>
              <Col>
                { this.props.details.event_name }
              </Col>
            </span>
          </Row>
          <Row className="score-row">
            <AllianceScore
              alliance="red"
              img="game/wide-white.png"
              score={match.score.red}
              stations={stations.filter(s => s.station.alliance === "red")}
              has_rp={has_rp}
            />
            <AllianceScore
              alliance="blue"
              img="tourney_logo_white.png"
              score={match.score.blue}
              stations={stations.filter(s => s.station.alliance === "blue")}
              has_rp={has_rp}
              reverse
            />
          </Row>
        </div>
      </div>
    }
  }
}