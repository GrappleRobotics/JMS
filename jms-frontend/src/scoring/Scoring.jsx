import React from "react";
import { Button, Container } from "react-bootstrap";
import { Link, Route, Switch, useRouteMatch } from "react-router-dom";


export class Scoring extends React.Component {
  renderNoScore() {

  }

  renderScore() {

  }

  render() {
    return <Container fluid>
      
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
      <Link to={`${url}/blue`}>
        <Button size="lg" variant="danger"> Red Alliance  </Button>
      </Link>
    </Route>
    <Route path={`${path}/blue`}>
      <Scoring {...props} alliance="blue" />
    </Route>
    <Route path={`${path}/red`}>
      <Scoring {...props} alliance="red" />
    </Route>
  </Switch>
}