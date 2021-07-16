import { faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BufferedFormControl from "components/elements/BufferedFormControl";
import React from "react";
import { Row, Col, Form } from "react-bootstrap";
import { nullIfEmpty } from "support/strings";

export default class ConfigureEvent extends React.Component {
  static eventKey() { return "configure_event"; }
  static tabName() { return "Configure Event"; }

  static needsAttention(event) {
    return event?.code === null || event?.event_name === null;
  }

  changeEventDetails = (changes) => {
    let newDetails = {
      ...this.props.event,
      ...changes
    };
    this.props.ws.send("event", "details", "update", newDetails);
  }

  render() {
    return <div>
      <h4> Configure Event Details </h4>
      <br />

      <Form>
        <Row>
          <Col md={4}>
            <Form.Label>Event Code</Form.Label>
            <BufferedFormControl 
              type="text"
              placeholder="2021myevent"
              value={this.props.event?.code}
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
              value={this.props.event?.event_name}
              onUpdate={v => this.changeEventDetails({ event_name: nullIfEmpty(v) })}
            />
          </Col>
        </Row>
      </Form>
    </div>
  }
}