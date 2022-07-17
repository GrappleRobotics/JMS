import { faCheck, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import _ from "lodash";
import React from "react";
import { Button, Col, Container, Row } from "react-bootstrap";
import { Link, Route, Routes } from "react-router-dom";
import { capitalise } from "support/strings";
import { otherAlliance, withVal } from "support/util";
import { WebsocketComponent } from "support/ws-component";
import { Alliance, AllianceStation, ArenaAccessRestriction, LoadedMatch, Penalties, SnapshotScore, ScoreUpdate, EndgamePointType, ArenaState } from "ws-schema";

type RefereePanelState = {
  match?: LoadedMatch,
  stations: AllianceStation[],
  access?: ArenaAccessRestriction,
  state?: ArenaState
}

abstract class RefereePanelBase<P={}> extends WebsocketComponent<P, RefereePanelState> {
  readonly state: RefereePanelState = { stations: [] };

  componentDidMount = () => this.handles = [
    this.listen("Arena/Alliance/CurrentStations", "stations"),
    this.listen("Arena/Match/Current", "match"),
    this.listen("Arena/Access/Current", "access"),
    this.listen("Arena/State/Current", "state")
  ]

  updateScore(alliance: Alliance, update: ScoreUpdate) {
    this.send({ Arena: { Match: { ScoreUpdate: { alliance, update } } } })
  }

  FoulsComponent = (props: {alliance: Alliance, score: SnapshotScore}) => {
    const { alliance, score } = props;
    const { live, derived } = score;

    const categories: { key: keyof Penalties, name: string }[] = [
      { key: 'fouls', name: "FOUL" },
      { key: 'tech_fouls', name: "TECHNICAL FOUL" }
    ];

    return <React.Fragment>
      {
        categories.map(category => <Col className="penalty-category" data-alliance={alliance}>
          <Row>
            <Col className="penalty-count"> { live.penalties[category.key] } </Col>
          </Row>
          <Row>
            <Col>
              <Button
                className="btn-block btn-penalty"
                data-penalty-type={category.key}
                variant={`${alliance}`}
                onClick={() => this.updateScore(alliance, { Penalty: { [category.key]: 1 } })}
              >
                {category.name}
              </Button>
              <Button
                className="btn-block btn-penalty"
                data-penalty-type={category.key}
                variant="secondary"
                onClick={() => this.updateScore(alliance, { Penalty: { [category.key]: -1 } })}
              >
                SUBTRACT
              </Button>
            </Col>
          </Row>
        </Col>)
      }
    </React.Fragment>
  }

  abstract renderIt(): React.ReactNode;
  abstract renderWaiting(): React.ReactNode;

  render() {
    return <Container fluid>
      {
        (this.state.match?.score != null && this.state.stations.length > 0) ? this.renderIt() : this.renderWaiting()
      }
    </Container>
  }
}

type RefereeTeamCardProps = {
  idx: number,
  station: AllianceStation,
  score: SnapshotScore,
  update: ( data: ScoreUpdate ) => void,
  endgame: boolean
};

const ENDGAME_MAP: { [K in EndgamePointType]: string } = {
  None: "None",
  Low: "Low",
  Mid: "Mid",
  High: "High",
  Traversal: "Trav."
};

export class RefereeTeamCard extends React.PureComponent<RefereeTeamCardProps> {
  render() {
    const { idx, station, score, update, endgame } = this.props;
    const alliance = station.station.alliance;

    const has_taxi = score.live.taxi[idx];

    return withVal(station.team, team => <Col className="referee-station" data-alliance={alliance}>
      <Row>
        <Col className="referee-station-team" md="auto"> { team } </Col>
        <Col>
          <Button
            className="btn-block referee-station-score"
            data-score-type="taxi"
            data-score-value={has_taxi}
            onClick={() => update( { Taxi: { station: idx, crossed: !has_taxi } } )}
          >
            {
              has_taxi ? <React.Fragment> AUTO TAXI OK &nbsp; <FontAwesomeIcon icon={faCheck} />  </React.Fragment>
                : <React.Fragment> NO AUTO TAXI &nbsp; <FontAwesomeIcon icon={faTimes} /> </React.Fragment>
            }
          </Button>
        </Col>
      </Row>
      <Row>
          <Col>
            <EnumToggleGroup
              name={`${team}-endgame`}
              className="referee-station-score"
              data-score-type="endgame"
              data-score-value={score.live.endgame[idx]}
              value={score.live.endgame[idx]}
              values={_.keys(ENDGAME_MAP) as EndgamePointType[]}
              names={_.values(ENDGAME_MAP)}
              onChange={v => update({ Endgame: { station: idx, endgame: v } })}
              // disabled={!endgame}
            />
          </Col>
      </Row>
    </Col>) || <Col />
  }
}

type AllianceRefereeProps = {
  alliance: Alliance,
  position: "near" | "far"
};

export class AllianceReferee extends RefereePanelBase<AllianceRefereeProps> {

  renderWaiting() {
    return <React.Fragment>
      <h3> Waiting for Scorekeeper... </h3>
      <i className="text-muted"> { capitalise(this.props.alliance) } Alliance Referee </i>
    </React.Fragment>
  }

  renderIt() {
    const match = this.state.match!;
    const alliance = this.props.alliance;
    const other_alliance = otherAlliance(alliance);
    
    const score = match.score[alliance];
    const other_score = match.score[other_alliance];

    const flip = this.props.position === "far";

    const stations = this.state.stations.filter(s => s.station.alliance === this.props.alliance);

    return <React.Fragment>
      <Row className="mb-3">
        <Col>
          <h3 className="mb-0"> { match.match_meta.name } </h3>
          <i className="text-muted"> { capitalise(alliance) } Alliance Referee </i>
        </Col>
        <Col className="text-end">
          <h3 className="text-muted"> { match.state || "--" } &nbsp; { match?.remaining_time?.secs }s </h3>
        </Col>
      </Row>
      <Row>
        <this.FoulsComponent
          alliance={flip ? "red" : "blue"}
          score={match.score[flip ? "red" : "blue"]}
        />
        <this.FoulsComponent
          alliance={flip ? "blue" : "red"}
          score={match.score[flip ? "blue" : "red"]}
        />
      </Row>
      <Row>
        {
          stations.map((station, i) => <RefereeTeamCard
            idx={i}
            station={station}
            score={score}
            update={(data) => this.updateScore(alliance, data)}
            endgame={match?.endgame || false}
          />)
        }
      </Row>
    </React.Fragment>
  }
}

export class HeadReferee extends RefereePanelBase {
  renderTopBar = () => {
    const { match, state, access } = this.state;

    const canChangeAccess = state != null && !(state.state === "MatchArmed" || state.state === "MatchPlay");

    return <React.Fragment>
      <Row className="mb-3">
        <Col>
          <h3 className="mb-0"> { match?.match_meta?.name || "Waiting for Scorekeeper..." } </h3>
          <h4 className="text-muted"> { match?.state || "--" } &nbsp; { match?.remaining_time?.secs }s </h4>
        </Col>
        <Col md="auto" className="head-ref-field-ax">
          <Button
            variant="purple"
            size="lg"
            onClick={() => this.send({ Arena: { Access: { Set: "ResetOnly" } } })}
            disabled={!canChangeAccess || access === "ResetOnly"}
          >
            FIELD RESET
          </Button>

          <Button
            variant="good"
            size="lg"
            onClick={() => this.send({ Arena: { Access: { Set: "ResetOnly" } } })}
            disabled={!canChangeAccess || access === "Teams"}
          >
            TEAMS ON FIELD
          </Button>

          <Button
            variant="primary"
            size="lg"
            onClick={() => this.send({ Arena: { Access: { Set: "NoRestriction" } } })}
            disabled={!canChangeAccess || access === "NoRestriction"}
          >
            NORMAL
          </Button>
        </Col>
      </Row>
    </React.Fragment>
  }

  renderWaiting() {
    return this.renderTopBar()
  }

  renderIt() {
    let match = this.state.match!;
    let { score } = match;

    return <React.Fragment>
      { this.renderTopBar() }
      <Row>
        <this.FoulsComponent
          alliance="blue"
          score={score.blue}
        />
        <this.FoulsComponent
          alliance="red"
          score={score.red}
        />
      </Row>
    </React.Fragment>
  }
}

class RefereeSelector extends React.PureComponent {
  render() {
    return <Col className="referee-selector-container">
      <Row>
        <h3 className="text-center mb-4"> Referee Selection </h3>
      </Row>
      <Row className="referee-selector-row">
        <Col className="referee-selector-image-container">
          {
            [ "blue", "red" ].map(alliance => [ "near", "far" ].map(position => (
              <Link to={`${alliance}/${position}`}>
                <Button className="referee-selector-btn" data-alliance={alliance} data-position={position}>
                  { capitalise(alliance) } { capitalise(position) }
                </Button>
              </Link>
            )))
          }

          <Link to="head">
            <Button className="referee-selector-btn" data-head-referee="true">
              Head Referee
            </Button>
          </Link>

          <img className="referee-selector-image" src="/img/game/field.png" />
        </Col>
      </Row>
    </Col>
  }
}

export function RefereeRouter() {
  return <Routes>
    <Route path="/" element={ <RefereeSelector /> } />
    <Route path="blue/near" element={ <AllianceReferee alliance="blue" position="near" /> } />
    <Route path="blue/far" element={ <AllianceReferee alliance="blue" position="far" /> } />
    <Route path="red/near" element={ <AllianceReferee alliance="red" position="near" /> } />
    <Route path="red/far" element={ <AllianceReferee alliance="red" position="far" /> } />
    <Route path="head" element={ <HeadReferee /> } />
  </Routes>
}
