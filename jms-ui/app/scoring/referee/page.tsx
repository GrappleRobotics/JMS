"use client"
import { useRouter } from "next/navigation";
import { Button, Col, Container, Row } from "react-bootstrap";

export default function RefereeSelector() {
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
          className="btn-block btn-full my-2"
          variant="purple"
          size="lg"
          onClick={() => router.push("/scoring/referee/head-referee")}
        >
          HEAD REFEREE
        </Button>
      </Col>
    </Row>
    <Row>
      <Col>
        <Button
          className="btn-block btn-full my-2"
          variant="blue"
          size="lg"
          onClick={() => router.push("/scoring/referee/blue/near")}
        >
          BLUE NEAR
        </Button>
        <Button
          className="btn-block btn-full my-2"
          variant="blue"
          size="lg"
          onClick={() => router.push("/scoring/referee/blue/far")}
        >
          BLUE FAR
        </Button>
      </Col>
      <Col>
        <Button
          className="btn-block btn-full my-2"
          variant="red"
          size="lg"
          onClick={() => router.push("/scoring/referee/red/near")}
        >
          RED NEAR
        </Button>
        <Button
          className="btn-block btn-full my-2"
          variant="red"
          size="lg"
          onClick={() => router.push("/scoring/referee/red/far")}
        >
          RED FAR
        </Button>
      </Col>
    </Row>
  </Container>
}