import { faInfoCircle, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BufferedFormControl from "components/elements/BufferedFormControl";
import EditableFormControl from "components/elements/EditableFormControl";
import React from "react";
import { Row, Col, Form, Button } from "react-bootstrap";
import { nullIfEmpty } from "support/strings";

export default class ConfigureEvent extends React.Component {
  static eventKey() { return "configure_event"; }
  static tabName() { return "Configure Event"; }

  static needsAttention(d) {
    return d.details?.event_name === null;
  }

  constructor(props) {
    super(props);

    this.state = {
      newWebcast: null
    }
  }

  changeEventDetails = (changes) => {
    let newDetails = {
      ...this.props.details,
      ...changes
    };
    this.props.ws.send("event", "details", "update", newDetails);
  }

  submitNewWebcast = () => {
    this.changeEventDetails({
      webcasts: [ ...this.props.details.webcasts, this.state.newWebcast ]
    });

    this.setState({ newWebcast: null })
  }

  deleteWebcast = (i) => {
    let webcasts = this.props.details.webcasts;
    webcasts.splice(i, 1);
    this.changeEventDetails({
      webcasts: webcasts
    });
  }

  render() {
    return <div>
      <h4> Configure Event Details </h4>
      <br />

      <Form>
        <Row>
          <Col md={4}>
            <Form.Label>Event Code <span className="text-muted">(Optional)</span></Form.Label>
            <BufferedFormControl 
              type="text"
              placeholder="2021myevent"
              value={this.props.details?.code}
              onUpdate={v => this.changeEventDetails({ code: nullIfEmpty(v) })}
            />
            <Form.Text className="text-muted">
              <FontAwesomeIcon icon={faInfoCircle} /> The event code is usually provided by <i>FIRST</i>
            </Form.Text>
          </Col>
          <Col>
            <Form.Label>Event Name</Form.Label>
            <BufferedFormControl
              type="text"
              placeholder="My Really Groovy Robotics Event"
              value={this.props.details?.event_name}
              onUpdate={v => this.changeEventDetails({ event_name: nullIfEmpty(v) })}
            />
          </Col>
        </Row>
        <Row className="my-3">
          <Col>
            <Form.Label> Webcast URLs (TBA) <span className="text-muted">(Optional)</span> </Form.Label>
            <Form.Control
              type="text"
              size="sm"
              value={ this.state.newWebcast || "" }
              placeholder="New Webcast URL, e.g. https://www.youtube.com/watch?v=dQw4w9WgXcQ"
              onChange={e => this.setState({ newWebcast: nullIfEmpty(e.target.value) })}
              onKeyDown={e => {
                if (e.key === 'Enter')
                  this.submitNewWebcast()
              }}
            />
            {
              this.props.details?.webcasts?.map((wc, i) => <Row className="my-2">
                <Col md="auto">
                  <Button
                    size="sm"
                    variant="danger"
                    onClick={() => this.deleteWebcast(i)}
                  >
                    <FontAwesomeIcon icon={faTimes} /> &nbsp; Delete
                  </Button>
                </Col>
                <Col>
                  <a href={wc} target="_blank">{ wc }</a>
                </Col>
              </Row>)
            }
          </Col>
        </Row>
      </Form>
    </div>
  }
}