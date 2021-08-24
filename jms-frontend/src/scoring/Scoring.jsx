import { faCheck, faMinus, faPlus, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BooleanToggleGroup from "components/elements/BooleanToggleGroup";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import React from "react";
import { Button, Card, Col, Container, Row, ToggleButton, ToggleButtonGroup } from "react-bootstrap";
import { TypeaheadInputMulti } from "react-bootstrap-typeahead";
import { Link, Route, Switch, useRouteMatch } from "react-router-dom";

export class ScoringAlliance extends React.Component {
  constructor(props) {
    super(props);

    props.ws.subscribe("arena", "match");
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
    return <Col>
      <h3> Waiting for Scorekeeper... </h3>
      <i className="text-muted"> { this.props.alliance } Alliance Scorer </i>
    </Col>
  }

  ButtonPair = (props) => {
    const { value, name, variant, reversed, onChange } = props;

    const cols = [
      <Col md={3}>
        <Button
          // @ts-ignore
          size="xl"
          variant="secondary"
          block
          onClick={ () => onChange(-1) }
        >
          -1
        </Button>
      </Col>,
      <Col>
        <Button
          // @ts-ignore
          size="xl"
          variant={variant}
          block
          onClick={ () => onChange(1) }
        >
          <span className="current-value"> { value } </span> <br />
          { name.toUpperCase() }
          {/* { name.toUpperCase() } */}
        </Button>
      </Col>
    ]

    return <Row className="scorer-button-pair" data-name={name}>
      { reversed ? cols.reverse() : cols }
    </Row>
  }

  ScorerCol = (props) => {
    const { mode, reversed, live } = props;

    const power_cells_scores = live.power_cells[mode.toLowerCase()];

    const that = this;

    return <Col className="scorer-col">
      <h4> { mode.toUpperCase() } </h4>
      <that.ButtonPair
        value={power_cells_scores.inner}
        name="Inner"
        variant="success"
        reversed={reversed}
        onChange={(offset) => this.updateScore("PowerCell", { auto: mode == "Auto", inner: offset })}
      />
      <br />
      <that.ButtonPair
        value={power_cells_scores.outer}
        name="Outer"
        variant="primary"
        reversed={reversed}
        onChange={(offset) => this.updateScore("PowerCell", { auto: mode == "Auto", outer: offset })}
      />
      <Row className="grow"> <Col /> </Row>
      <that.ButtonPair
        value={power_cells_scores.bottom}
        name="Bottom"
        variant="danger"
        reversed={reversed}
        onChange={(offset) => this.updateScore("PowerCell", { auto: mode == "Auto", bottom: offset })}
      />
    </Col>
  }

  renderScore() {
    const alliance = this.props.alliance.toLowerCase();
    const match = this.props.arena.match;
    const score = this.props.arena.match.score[alliance];
    const { live, derived } = score;

    return <Col className="scorer-container">
      <Row className="mb-3">
        <Col>
          <h3 className="mb-0"> { match.match.name } </h3>
          <i className="text-muted"> { this.props.alliance } Alliance Scorer </i>
        </Col>
        <Col className="text-right">
          <h3 className="text-muted"> { match.state } &nbsp; { match.remaining_time.secs }s </h3>
        </Col>
      </Row>
      <Row className="scorer-interface">
        <this.ScorerCol
          mode="Auto"
          live={live}
          reversed
        />
        <Col className="scorer-img" md="auto">
          <img src="/img/game/power-port.png" />
        </Col>
        <this.ScorerCol
          mode="Teleop"
          live={live}
        />
      </Row>
    </Col>
  }

  render() {
    return (this.props.arena?.match?.score ) ? this.renderScore() : this.renderNoScore();
  }
}

export function ScoringRouter(props) {
  let { path, url } = useRouteMatch();

  return <Switch>
    <Route exact path={path}>
      <Container>
        <h3 className="mb-4"> Scorer Selection </h3>
        <Link to={`${url}/blue`}>
          <Button size="lg" variant="primary"> Blue Alliance  </Button>
        </Link> &nbsp;
        <Link to={`${url}/red`}>
          <Button size="lg" variant="danger"> Red Alliance  </Button>
        </Link>
      </Container>
    </Route>
    <Route path={`${path}/blue`}>
      <ScoringAlliance {...props} alliance="Blue" />
    </Route>
    <Route path={`${path}/red`}>
      <ScoringAlliance {...props} alliance="Red" />
    </Route>
  </Switch>
}