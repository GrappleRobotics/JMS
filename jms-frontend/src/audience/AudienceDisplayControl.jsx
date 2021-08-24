import React from "react";
import { Col, Container, Row, Button, Modal, Form } from "react-bootstrap";

export default class AudienceDisplayControl extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      showCustomMessage: false,
      customMessage: ""
    };

    props.ws.subscribe("event", "awards");
  }
  
  send = (scene, params) => {
    this.props.ws.send("arena", "audience_display", "set", {
      scene: scene,
      params: params
    })
  }

  custom = (msg) => {
    this.send("CustomMessage", msg);
    this.setState({
      showCustomMessage: false,
      customMessage: ""
    });
  }

  customMessageModal = () => {
    return <Modal
      centered
      size="lg"
      show={this.state.showCustomMessage} 
      onHide={() => this.setState({ showCustomMessage: false })}
      animation={false}
    >
      <Modal.Header>
        <Modal.Title> Custom Message </Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <Form.Control
          className="my-3"
          type="text"
          size="lg"
          autoFocus
          placeholder="Your custom message. This will show the whole audience!"
          onChange={e => this.setState({ customMessage: e.target.value })}
          onKeyDown={e => (e.key === 'Enter') && this.custom(this.state.customMessage)}
          value={this.state.customMessage}
        />
        <Button
          variant="success"
          onClick={() => this.custom(this.state.customMessage)}
          block
          size="lg"
        >
          Show!
        </Button>
      </Modal.Body>
    </Modal>
  }

  render() {
    const sections = [
      {
        title: "General Purpose",
        scenes: [
          { name: "Field", scene: "Field" },
          { name: "Custom...", scene: "Custom Message", variant: "warning", onClick: () => this.setState({ showCustomMessage: true }) }
        ]
      },
      {
        title: "Match Control",
        scenes: [
          { name: "Match Preview", scene: "MatchPreview", variant: "warning" },
          { name: "Match Play", scene: "MatchPlay", variant: "success" },
          { name: "Match Results (latest)", scene: "MatchResults", variant: "danger" },
        ]
      },
      {
        title: "Alliance Selections",
        scenes: [
          { name: "Alliance Selections", scene: "AllianceSelection" }
        ]
      }
    ];

    return <Container className="audience-control">
      <h2> Audience Display Control </h2>
      <p> If displays are not yet ready to display data (e.g. match is not loaded), displays will default to a blank 
        field view until data is ready. </p>
      <br />

      {
        sections.map(section => <Row className="mb-4">
          <Col>
            <h3> { section.title } </h3>
            <div className="ml-4">
              {
                section.scenes.map(scene => <React.Fragment>
                  <Button
                    className="m-1"
                    size="lg"
                    block
                    onClick={() => scene.onClick ? scene.onClick() : this.send(scene.scene, scene.params || null)}
                    variant={scene.variant || "primary"}
                  >
                    { scene.name }
                  </Button> <br />
                </React.Fragment>)
              }
            </div>
          </Col>
        </Row>)
      }

      <Row className="mb-4">
        <Col>
          <h3> Awards </h3>
          <div className="ml-4">
            {
              this.props.event?.awards?.map(award => <React.Fragment>
                <Button
                  className="m-1 px-5 award-btn"
                  onClick={() => this.send("Award", award.id)}
                  disabled={award.recipients.length == 0}
                >
                  { award.name }
                </Button>
              </React.Fragment>)
            }
          </div>
        </Col>
      </Row>

      { this.customMessageModal() }

    </Container>
  }
}