import React from "react";
import { Col, Row } from "react-bootstrap";
import AudienceCard from "./AudienceCard";
import BaseAudienceScene from "./BaseAudienceScene";

type CustomMessageProps = {
  msg: string
};

export default class AudienceSceneCustomMessage extends BaseAudienceScene<CustomMessageProps> {
  show = (props: CustomMessageProps) => {
    const { msg } = props;

    return <AudienceCard event_name={this.props.details.event_name}>
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