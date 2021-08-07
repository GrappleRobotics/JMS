import { faCheck, faMinus, faPlus, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BooleanToggleGroup from "components/elements/BooleanToggleGroup";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import React from "react";
import { Button, Card, Col, Container, Row, ToggleButton, ToggleButtonGroup } from "react-bootstrap";
import { TypeaheadInputMulti } from "react-bootstrap-typeahead";
import { Link, Route, Switch, useRouteMatch } from "react-router-dom";

class ScoreTeam extends React.PureComponent {
  render() {
    let { station, livescore, idx, update } = this.props;
    let alliance = station.station.alliance.toLowerCase();
    let team = station.team;

    return <Card className="scorer-card" data-alliance={alliance} border={`alliance-${alliance}`}>
      <Card.Header className="h4"> { team || <i className="text-muted"> Unoccupied </i> } </Card.Header>
      {
        !!!team ? <Card.Body> <i className="text-muted"> Unoccupied </i>  </Card.Body> 
          : <Card.Body className="text-muted m-0 py-1">
            <Row className="my-3">
              <Col md={5}> Auto Cross </Col>
              <Col>
                <BooleanToggleGroup 
                  name={`${TypeaheadInputMulti}-auto-cross`} 
                  value={livescore.initiation_line_crossed[idx]} 
                  onChange={v => update("Initiation", { station: idx, crossed: v })}
                  size="md"
                />
              </Col>
            </Row>
            <Row className="my-2">
              <Col> End Game </Col>
            </Row>
            <Row className="mb-3">
              <Col>
                <EnumToggleGroup
                  name={`${team}-endgame`}
                  value={livescore.endgame[idx]}
                  onChange={v => update("Endgame", { station: idx, endgame: v })}
                  values={["None", "Park", "Hang"]}
                  outline
                  variant="light"
                />
              </Col>
            </Row>
          </Card.Body>
      }
      
    </Card>
  }
}

export class ScoringAlliance extends React.Component {
  constructor(props) {
    super(props);

    props.ws.subscribe("arena", "match");
    props.ws.subscribe("arena", "stations");
  }

  updateScore(field, data) {
    console.log(field, data);
    this.props.ws.send("arena", "match", "scoreUpdate", {
      alliance: this.props.alliance,
      update: {
        [field]: data
      }
    });
  }

  renderNoScore() {
    return <React.Fragment>
      <h3> Waiting for Scorekeeper... </h3>
      <i className="text-muted"> { this.props.alliance } Alliance Scorer </i>
    </React.Fragment>
  }

  renderScore() {
    let alliance = this.props.alliance.toLowerCase();
    let match_state = this.props.arena.match.state;
    let score = this.props.arena.match.score[alliance];
    let { live, derived } = score;
    let stations = this.props.arena.stations.filter(s => s.station.alliance == this.props.alliance);

    return <React.Fragment>
      <Row className="mb-3">
        <Col>
          <h3 className="mb-0"> { this.props.arena.match.match.name } </h3>
          <i className="text-muted"> { this.props.alliance } Alliance Scorer </i>
        </Col>
      </Row>
      <Row className="mb-4">
        {
          stations.map((station, i) => <Col>
            <ScoreTeam
              idx={i}
              station={station}
              livescore={live}
              update={(field, data) => this.updateScore(field, data)}
            />
          </Col>)
        }
      </Row>
      <Row>
        {
          ["Auto", "Teleop"].map(mode => <Col>
            <Card className="scorer-card" border={ match_state == mode ? `alliance-${alliance}` : `dark` }>
              <Card.Header className="h5"> { mode } </Card.Header>
              <Card.Body className="text-muted h6 m-0">
                {
                  ["Inner", "Outer", "Bottom"].map(goal => <Row>
                    <Col md={4}> {goal} </Col>
                    <Col>
                      <Button variant="outline-success" onClick={ () => this.updateScore("PowerCell", { auto: mode == "Auto", [goal.toLowerCase()]: 1 }) }> 
                        <FontAwesomeIcon icon={faPlus} />
                      </Button>
                    </Col>
                    <Col md={2} className="p-0"> <h5 className="text-center"> { live.power_cells[mode.toLowerCase()][goal.toLowerCase()] } </h5> </Col>
                    <Col>
                      <Button variant="outline-danger" onClick={ () => this.updateScore("PowerCell", { auto: mode == "Auto", [goal.toLowerCase()]: -1 }) }>
                        <FontAwesomeIcon icon={faMinus} />
                      </Button>
                    </Col>
                  </Row>)
                }
              </Card.Body>
            </Card>
          </Col>)
        }
        <Col>
          <Card className="scorer-card" border={ match_state == "Complete" ? `alliance-${alliance}` : `dark` }>
            <Card.Header className="h5"> End Game </Card.Header>
            <Card.Body className="text-muted h6 m-0">
              <Row>
                <Col md={4}> Rung Level </Col>
                <Col>
                  <BooleanToggleGroup
                    name="rung-level"
                    value={live.rung_level}
                    onChange={v => this.updateScore("RungLevel", v)}
                    size="lg"
                  />
                </Col>
              </Row>
            </Card.Body>
          </Card>
        </Col>
      </Row>
    </React.Fragment>
  }

  render() {
    return <Container fluid>
      {
        (this.props.arena?.match?.score && this.props.arena?.stations ) ? this.renderScore() : this.renderNoScore()
      }
    </Container>
  }
}

export class ScoringCenter extends React.Component {
  constructor(props) {
    super(props);

    props.ws.subscribe("arena", "match");
    props.ws.subscribe("arena", "stations");
  }

  updateScore(alliance, field, data) {
    this.props.ws.send("arena", "match", "scoreUpdate", {
      alliance: alliance,
      update: {
        [field]: data
      }
    });
  }

  renderNoScore() {
    return <React.Fragment>
      <h3> Waiting for Scorekeeper... </h3>
      <i className="text-muted"> Center Field Scorer </i>
    </React.Fragment>
  }

  renderScoreForAlliance = (alliance) => {
    let match_state = this.props.arena.match.state;
    let score = this.props.arena.match.score[alliance.toLowerCase()];
    let { live, derived } = score;
    let stations = this.props.arena.stations.filter(s => s.station.alliance == alliance);

    return <Row className="mb-4">
      {
        stations.map((station, i) => <Col>
          <ScoreTeam
            idx={i}
            station={station}
            livescore={live}
            update={(field, data) => this.updateScore(alliance, field, data)}
          />
        </Col>)
      }
      <Col md={2}>
        <Card className="scorer-card" border={ match_state == "Complete" ? `alliance-${alliance.toLowerCase()}` : `dark` }>
          <Card.Header className="h5"> End Game </Card.Header>
          <Card.Body className="text-muted h6 m-0">
            <Row>
              <Col> Rung Level </Col>
            </Row>
            <Row>
              <Col>
                <BooleanToggleGroup
                  name="rung-level"
                  value={live.rung_level}
                  onChange={v => this.updateScore(alliance, "RungLevel", v)}
                  size="lg"
                />
              </Col>
            </Row>
          </Card.Body>
        </Card>
      </Col>
    </Row>
  }

  renderScore() {
    return <React.Fragment>
      <Row className="mb-3">
        <Col>
          <h3 className="mb-0"> { this.props.arena.match.match.name } </h3>
          <i className="text-muted"> Center Field Scorer </i>
        </Col>
      </Row>
      { this.renderScoreForAlliance("Blue") }
      { this.renderScoreForAlliance("Red") }
    </React.Fragment>
  }

  render() {
    return <Container fluid>
      {
        (this.props.arena?.match?.score && this.props.arena?.stations ) ? this.renderScore() : this.renderNoScore()
      }
    </Container>
  }
}

export function ScoringRouter(props) {
  let { path, url } = useRouteMatch();

  return <Switch>
    <Route exact path={path}>
      <h3 className="mb-4"> Scorer Selection </h3>
      <Link to={`${url}/blue`}>
        <Button size="lg" variant="primary"> Blue Alliance  </Button>
      </Link> &nbsp;
      <Link to={`${url}/center`}>
        <Button size="lg" variant="warning"> Center Field </Button>
      </Link> &nbsp;
      <Link to={`${url}/red`}>
        <Button size="lg" variant="danger"> Red Alliance  </Button>
      </Link>
    </Route>
    <Route path={`${path}/blue`}>
      <ScoringAlliance {...props} alliance="Blue" />
    </Route>
    <Route path={`${path}/red`}>
      <ScoringAlliance {...props} alliance="Red" />
    </Route>
    <Route path={`${path}/center`}>
      <ScoringCenter {...props} />
    </Route>
  </Switch>
}