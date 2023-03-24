import { confirmModal, withConfirm } from "components/elements/Confirm";
import Paginate from "components/elements/Paginate";
import React from "react";
import { Button, Col, Container, Modal, Row } from "react-bootstrap";
import { WebsocketComponent } from "support/ws-component";
import { SerializedMatch, SupportTicket } from "ws-schema";

type DebugState = {
  tickets?: SupportTicket[],
  matches: SerializedMatch[]
};

export default class Debug extends WebsocketComponent<{ fta: boolean }, DebugState> {
  readonly state: DebugState = { matches: [] };

  componentDidMount = () => this.handles = [
    this.listen("Ticket/All", "tickets"),
    this.listen("Match/All", "matches")
  ];

  selectUnplayed = (cb: (id: string) => void) => {
    confirmModal("", {
      data: "",
      title: "Fill Random",
      render: (ok, cancel) => <React.Fragment>
        <Modal.Body>
          <Paginate itemsPerPage={10}>
            {
              this.state.matches.filter(m => m.ready && !m.played).map(m => (
                <Button className="btn-block m-1" onClick={() => ok(m.id!)}> { m.name } </Button>
              ))
            }
          </Paginate>
        </Modal.Body>
        <Modal.Footer>
          <Button onClick={cancel} variant="secondary"> Cancel </Button>
        </Modal.Footer>
      </React.Fragment>
    }).then(cb)
  }

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
              {/* <Button
                onClick={() => withConfirm(() => this.send({ Debug: { Match: { FillRandomScores: null } } }))}
                variant="danger"
              >
                Random Fill (all)
              </Button>
              <br />
              <br />
              <Button
                onClick={() => this.selectUnplayed(id => this.send({ Debug: { Match: { FillRandomScores: id } } }))}
                variant="danger"
              >
                Random Fill (select)
              </Button> */}
              <br />
              <br />
              <Button
                onClick={() => withConfirm(() => this.send({ Debug: { Match: "DeleteAll" } }))}
                variant="danger"
              >
                DELETE ALL (DANGER)
              </Button>
            </Col>
          </Row>
        </React.Fragment>
      }
      <Row>
        <Col>
          <code> { JSON.stringify(this.state.tickets || {}, null, 2) } </code>
        </Col>
      </Row>
    </Container>
  }
}
