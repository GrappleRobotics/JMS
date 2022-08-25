import React from "react";
import { Col, Row } from "react-bootstrap";

type AudienceCardProps = {
  className?: string,
  event_name?: string | null,
  children: React.ReactNode
};

export default class AudienceCard extends React.PureComponent<AudienceCardProps> {
  render() {
    const { className, event_name, children } = this.props;

    return <div className={`audience-card ${className || ""}`}>
      <div className="audience-card-inner">
        <Row className="event-name">
          <Col md="auto">
            <img src="/img/game/game.png" />
          </Col>
          <Col className="d-flex align-items-center justify-content-center">
            { event_name || "" }
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