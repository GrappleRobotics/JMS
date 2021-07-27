import { faCheck, faMinus, faPlus, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BooleanToggleGroup from "components/elements/BooleanToggleGroup";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import React from "react";
import { Button, Card, Col, Container, Row, ToggleButton, ToggleButtonGroup } from "react-bootstrap";
import { Link, Route, Switch, useRouteMatch } from "react-router-dom";

export class Scoring extends React.Component {
  updateScore(field, data) {
    this.props.ws.send("arena", "match", "scoreUpdate", {
      alliance: this.props.alliance,
      update: {
        [field]: data
      }
    });
  }

  renderNoScore() {
    return <h3> Waiting for Scorekeeper... </h3>
  }

  renderScore() {
    let alliance = this.props.alliance.toLowerCase();
    let match_state = this.props.arena.match.state;
    let score = this.props.arena.match.score[alliance];
    let { live, derived } = score;
    let teams = this.props.arena.stations.filter(s => s.station.alliance == this.props.alliance).map(s => s.team);

    return <React.Fragment>
      <Row className="mb-3">
        <Col>
          <h3> { this.props.arena.match.match.name } </h3>
        </Col>
      </Row>
      <Row className="mb-4">
        {
          teams.map((t, i) => <Col>
            <Card className="scorer-card" data-alliance={alliance} border={`alliance-${alliance}`}>
              <Card.Header className="h4"> { t || <i className="text-muted"> Unoccupied </i> } </Card.Header>
              {
                !!!t ? <Card.Body> <i className="text-muted"> Unoccupied </i>  </Card.Body> 
                 : <Card.Body className="text-muted m-0">
                    <Row>
                      <Col md={3}> Auto Cross </Col>
                      <Col>
                        <BooleanToggleGroup 
                          name={`${t}-auto-cross`} 
                          value={live.initiation_line_crossed[i]} 
                          onChange={v => this.updateScore("Initiation", { station: i, crossed: v })}
                          size="lg"
                        />
                      </Col>
                    </Row>
                    <Row>
                      <Col md={3}> End Game </Col>
                      <Col>
                        <EnumToggleGroup
                          name={`${t}-endgame`}
                          value={live.endgame[i]}
                          onChange={v => this.updateScore("Endgame", { station: i, endgame: v })}
                          values={["None", "Park", "Hang"]}
                          outline
                          variant="light"
                        />
                      </Col>
                    </Row>
                  </Card.Body>
              }
              
            </Card>
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
                    <Col> <h4 className="text-center"> { live.power_cells[mode.toLowerCase()][goal.toLowerCase()] } </h4> </Col>
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
        this.props.arena?.match?.score ? this.renderScore() : this.renderNoScore()
      }
    </Container>
  }
}

export function ScoringRouter(props) {
  let { path, url } = useRouteMatch();

  return <Switch>
    <Route exact path={path}>
      <Link to={`${url}/blue`}>
        <Button size="lg" variant="primary"> Blue Alliance  </Button>
      </Link> &nbsp;
      <Link to={`${url}/red`}>
        <Button size="lg" variant="danger"> Red Alliance  </Button>
      </Link>
    </Route>
    <Route path={`${path}/blue`}>
      <Scoring {...props} alliance="Blue" />
    </Route>
    <Route path={`${path}/red`}>
      <Scoring {...props} alliance="Red" />
    </Route>
  </Switch>
}