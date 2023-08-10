import { EventDetails } from "@/app/ws-schema";
import AudienceCard from "../card";
import { Col, Row } from "react-bootstrap";

export default function MessageScene({ message, eventDetails }: { message: string, eventDetails: EventDetails }) {
  return <AudienceCard event_name={eventDetails.event_name}>
    <Row>
      <Col className="audience-card-title" md="auto">
        Event Message
      </Col>
    </Row>
    <Row className="custom-message">
      <Col>
        { message }
      </Col>
    </Row>
  </AudienceCard>
}