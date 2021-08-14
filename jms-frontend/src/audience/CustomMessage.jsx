import React from "react";
import { Col, Row } from "react-bootstrap";
import AudienceCard from "./AudienceCard";

export default class CustomMessage extends React.PureComponent {
  render() {
    const { event, msg } = this.props;

    return <AudienceCard event={event}>
      <Row>
        <Col className="audience-card-title" md="auto">
          Event Message
        </Col>
      </Row>
      <Row className="custom-message">
        <Col>
          { msg }
        </Col>
      </Row>
    </AudienceCard>
  }
}