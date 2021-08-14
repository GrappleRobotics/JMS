import React from "react";
import { Col, Row } from "react-bootstrap";

export default class AudienceCard extends React.PureComponent {
  render() {
    const { className, event, children } = this.props;

    return <div className={`audience-card ${className}`}>
      <div className="audience-card-inner">
        <Row className="event-name">
          <Col md="auto">
            <img src="/img/game/wide-black.png" />
          </Col>
          <Col className="d-flex align-items-center justify-content-center">
            { event.details.event_name || "" }
          </Col>
          <Col md="auto">
            <img src="/img/tourney_logo.png" />
          </Col>
        </Row>

        { children }
      </div>
    </div>
  }
}