import { faCheck, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import React from "react";
import { Button, Col, Container, Row } from "react-bootstrap";
import { Link, Route, Switch, useRouteMatch } from "react-router-dom";
import { withVal } from "support/util";

class RefereePanel extends React.PureComponent {
  constructor(props) {
    super(props);

    props.ws.subscribe("arena", "match");
    props.ws.subscribe("arena", "stations");
    props.ws.subscribe("arena", "access");
  }

  updateScore(alliance, field, data) {
    console.log(field, data);
    this.props.ws.send("arena", "match", "scoreUpdate", {
      alliance: alliance.toLowerCase() == "blue" ? "Blue" : "Red",  // undo toLowerCase below
      update: {
        [field]: data
      }
    });
  }

  AllianceFouls = (props) => {
    let { score, alliance } = props;
    let { live, derived } = score;

    const categories = [
      { key: 'fouls', name: "FOUL" },
      { key: 'tech_fouls', name: "TECHNICAL FOUL" }
    ]

    return <React.Fragment>
      {
        categories.map(category => <Col className="penalty-category" data-alliance={alliance}>
          <Row>
            <Col className="penalty-count"> { live.penalties[category.key] } </Col>
          </Row>
          <Row>
            <Col>
              <Button
                block
                variant={`alliance-${alliance}`}
                // @ts-ignore
                size="xl"
                onClick={() => this.updateScore(alliance, "Penalty", { [category.key]: 1 })}
              >
                {category.name}
              </Button>
              <Button
                block
                variant="secondary"
                // @ts-ignore
                size="xl"
                onClick={() => this.updateScore(alliance, "Penalty", { [category.key]: -1 })}
              >
                SUBTRACT
              </Button>
            </Col>
          </Row>
        </Col>)
      }
    </React.Fragment>
  }

  render() {
    return <Container fluid>
      {
        // @ts-ignore
        (this.props.arena?.match?.score && this.props.arena?.stations) ? this.renderIt() : this.renderWaiting()
      }
    </Container>
  }
}

export class RefereeAlliance extends RefereePanel {
  TeamCard = (props) => {
    let { station, live, idx, update, endgame} = props;
    let alliance = station.station.alliance.toLowerCase();
    let team = station.team;
    let crossed = live.initiation_line_crossed[idx];

    return withVal(team, () => <Col className="referee-team" data-alliance={alliance}>
      <Row className="mb-3">
        <Col className="team" md="auto"> { team } </Col>
        <Col>
          <Button
            variant={ crossed ? "success" : "danger" }
            size="lg"
            block
            onClick={() => update("Initiation", { station: idx, crossed: !crossed })}
          >
            { 
              crossed ? <React.Fragment> AUTO CROSS OK &nbsp; <FontAwesomeIcon icon={faCheck} /> </React.Fragment>
                : <React.Fragment> NO AUTO CROSS &nbsp; <FontAwesomeIcon icon={faTimes} /> </React.Fragment>
            }
          </Button>
        </Col>
      </Row>
      <Row>
        <Col className="endgame-state">
          <EnumToggleGroup
            name={`${team}-endgame`}
            value={live.endgame[idx]}
            onChange={v => update("Endgame", { station: idx, endgame: v })}
            values={["None", "Park", "Hang"]}
            outline
            variant={
              live.endgame[idx] == "None" ? "light" : "success"
            }
            size="lg"
            disabled={!endgame}
          />
        </Col>
      </Row>
    </Col>) || <Col />
  }

  renderWaiting() {
    return <React.Fragment>
      <h3> Waiting for Scorekeeper... </h3>
      <i className="text-muted"> { this.props.alliance } Alliance Scorer </i>
    </React.Fragment>
  }

  renderIt() {
    let match = this.props.arena?.match;

    let alliance = this.props.alliance.toLowerCase();
    let other_alliance = alliance == "blue" ? "red" : "blue";

    let score = this.props.arena.match.score[alliance];
    let other_score = this.props.arena.match.score[other_alliance];

    let stations = this.props.arena.stations.filter(s => s.station.alliance == this.props.alliance);

    return <React.Fragment>
      <Row className="mb-3">
        <Col>
          <h3 className="mb-0"> { this.props.arena.match.match.name } </h3>
          <i className="text-muted"> { this.props.alliance } Alliance Referee </i>
        </Col>
        <Col className="text-end">
          <h3 className="text-muted"> { match?.state || "--" } &nbsp; { match?.remaining_time?.secs }s </h3>
        </Col>
      </Row>
      <Row>
        <this.AllianceFouls
          alliance={alliance}
          score={score}
        />
        <this.AllianceFouls
          alliance={other_alliance}
          score={other_score}
        />
      </Row>
      <Row>
        {
          stations.map((station, i) => <this.TeamCard
            idx={i}
            station={station}
            live={score.live}
            update={(field, data) => this.updateScore(alliance, field, data)}
            endgame={match?.endgame || false}
          />)
        }
      </Row>
    </React.Fragment>
  }
}

export class HeadReferee extends RefereePanel {
  renderTopBar = () => {
    let match = this.props.arena?.match;
    let state = this.props.arena?.state?.state;
    let access = this.props.arena?.access;

    return <React.Fragment>
      <Row className="mb-3">
        <Col>
          <h3 className="mb-0"> { this.props.arena?.match?.match?.name || "Waiting for Scorekeeper..." } </h3>
          <h4 className="text-muted"> { match?.state || "--" } &nbsp; { match?.remaining_time?.secs }s </h4>
        </Col>
        <Col md="auto" className="head-ref-field-ax">
          <Button
            variant="purple"
            size="lg"
            onClick={() => this.props.ws.send("arena", "access", "set", "ResetOnly")}
            disabled={state === "MatchArmed" || state === "MatchPlay" || access === "ResetOnly"}
          >
            FIELD RESET
          </Button>

          <Button
            variant="success"
            size="lg"
            onClick={() => this.props.ws.send("arena", "access", "set", "Teams")}
            disabled={state === "MatchArmed" || state === "MatchPlay" || access === "Teams"}
          >
            TEAMS ON FIELD
          </Button>

          <Button
            variant="primary"
            size="lg"
            onClick={() => this.props.ws.send("arena", "access", "set", "NoRestriction")}
            disabled={state === "MatchArmed" || state === "MatchPlay" || access === "NoRestriction"}
          >
            NORMAL
          </Button>
        </Col>
      </Row>
    </React.Fragment>
  }

  RungLevel = (props) => {
    let { score, alliance, endgame } = props;
    let rung_level = score.live.rung_level;

    return <Col>
      <Button
        variant={endgame ? (rung_level ? "success" : "danger") : "secondary"}
        // @ts-ignore
        size="xl"
        block
        onClick={v => this.updateScore(alliance, "RungLevel", !rung_level)}
        disabled={!endgame}
      >
        <FontAwesomeIcon icon={rung_level ? faCheck : faTimes} />
        &nbsp; &nbsp;
        {
          rung_level ? "RUNG LEVEL OK" : "RUNG NOT LEVEL"
        }
        &nbsp; &nbsp;
        <FontAwesomeIcon icon={rung_level ? faCheck : faTimes} />
      </Button>
    </Col>
  }

  renderWaiting() {
    return this.renderTopBar()
  }

  renderIt() {
    let match = this.props.arena.match;
    let { score, endgame } = match;

    return <React.Fragment>
      { this.renderTopBar() }
      <Row>
        <this.AllianceFouls
          alliance={"red"}
          score={score.red}
        />
        <this.AllianceFouls
          alliance={"blue"}
          score={score.blue}
        />
      </Row>
      <Row>
        <this.RungLevel
          alliance={"red"}
          score={score.red}
          endgame={endgame}
        />
        <this.RungLevel
          alliance={"blue"}
          score={score.blue}
          endgame={endgame}
        />
      </Row>
    </React.Fragment>
  }
}

export function RefereeRouter(props) {
  let { path, url } = useRouteMatch();

  return <Switch>
    <Route exact path={path}>
      <Container>
        <h3 className="mb-4"> Referee Selection </h3>
        <Link to={`${url}/blue`}>
          <Button size="lg" variant="primary"> Blue Alliance  </Button>
        </Link> &nbsp;
        <Link to={`${url}/head`}>
          <Button size="lg" variant="warning"> Head Referee </Button>
        </Link> &nbsp;
        <Link to={`${url}/red`}>
          <Button size="lg" variant="danger"> Red Alliance  </Button>
        </Link>
      </Container>
    </Route>
    <Route path={`${path}/blue`}>
      <RefereeAlliance {...props} alliance="Blue" />
    </Route>
    <Route path={`${path}/red`}>
      <RefereeAlliance {...props} alliance="Red" />
    </Route>
    <Route path={`${path}/head`}>
      <HeadReferee {...props} />
    </Route>
  </Switch>
}
