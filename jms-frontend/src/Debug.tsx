import React from "react";
import { Button, Col, Container, Row } from "react-bootstrap";
import { WebsocketComponent } from "support/ws-component";
import { TaggedResource } from "ws-schema";

type DebugState = {
  resources?: TaggedResource[]
};

export default class Debug extends WebsocketComponent<{ fta: boolean }, DebugState> {
  readonly state: DebugState = {};

  componentDidMount = () => this.handles = [
    this.listen("Resource/All", "resources")
  ];

  render() {
    let screen = window.screen;
    let root = document.documentElement;

    return <Container>
      <Row>
        <Col>
          <h2> Debug Information </h2>
        </Col>
      </Row>
      <Row>
        <Col>
          <h4> Device Details </h4>
          <p> Screen dimensions: { screen.width } x { screen.height } </p>
          <p> Screen dimensions (avail): { screen.availWidth } x { screen.availHeight } </p>
          <p> Device pixel ratio: { window.devicePixelRatio } </p>
          <p> Root Element Dimensions: { root.clientWidth } x { root.clientHeight } </p>
          <p> Window Inner Dimensions: { window.innerWidth } x { window.innerHeight } </p>
        </Col>
      </Row>
      {
        !this.props.fta ? <React.Fragment /> : <React.Fragment>
          <Row>
            <Col md="auto" className="mx-3">
              <h4> Tests </h4>
              <Button onClick={() => {
                this.transact<String>({ Debug: { ReplyTest: "Hello World!" } }, "Debug/ReplyTest")
                  .then(data => alert(JSON.stringify(data)))
                  .catch(reason => alert(`CATCH: ${reason}`))
              }}>
                Test Reply
              </Button>
            </Col>
            <Col md="auto" className="mx-3">
              <h4> Matches </h4>
              <Button
                onClick={() => this.send({ Debug: { Match: "FillRandomScores" } })}
                variant="danger"
              >
                Random Fill
              </Button>
              <br />
              <br />
              <Button
                onClick={() => this.send({ Debug: { Match: "DeleteAll" } })}
                variant="danger"
              >
                DELETE ALL (DANGER)
              </Button>
            </Col>
          </Row>
        </React.Fragment>
      }
    </Container>
  }
}