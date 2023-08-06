"use client"
import { useRouter } from "next/navigation";
import { Button, Col, Container, Row } from "react-bootstrap";

export default function ScoringSelector() {
  const router = useRouter();

  return <Container>
    <Row className="my-3">
      <Col>
        <h3> Select Alliance </h3>
      </Col>
    </Row>
    <Row>
      <Col>
        <Button
          className="btn-block btn-full"
          variant="blue"
          size="lg"
          onClick={() => router.push("/scoring/blue")}
        >
          BLUE
        </Button>
      </Col>
      <Col>
        <Button
          className="btn-block btn-full"
          variant="red"
          size="lg"
          onClick={() => router.push("/scoring/red")}
        >
          RED
        </Button>
      </Col>
    </Row>
  </Container>
}