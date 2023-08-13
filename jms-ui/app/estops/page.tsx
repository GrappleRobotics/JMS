"use client";
import { useRouter } from "next/navigation";
import React from "react";
import { Button, Col, Container, Row } from "react-bootstrap";

export default function EstopIndex() {
  const router = useRouter();

  return <Container>
    <h3 className="my-4"> Select Station </h3>
    <Row>
      <Col>
        { [1, 2, 3].map(station => <Button key={`blue${station}`} variant="blue" size="lg" className="btn-block my-2" onClick={() => router.push(`/estops/blue/${station}`)}>
          BLUE { station }
        </Button>) }
      </Col>
      <Col>
        { [1, 2, 3].map(station => <Button key={`red${station}`} variant="red" size="lg" className="btn-block my-2" onClick={() => router.push(`/estops/red/${station}`)}>
          RED { station }
        </Button>) }
      </Col>
    </Row>
  </Container>
}