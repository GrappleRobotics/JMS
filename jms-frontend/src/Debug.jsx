import React from "react";
import { Col, Container, Row } from "react-bootstrap";

export default class Debug extends React.PureComponent {
  render() {
    let screen = window.screen;
    let root = document.documentElement;

    return <Container>
      <Row>
        <Col>
          <h2> Debug Information </h2>
        </Col>
      </Row>
      <Row>
        <Col>
          <h4> Device Details </h4>
          <p> Screen dimensions: { screen.width } x { screen.height } </p>
          <p> Screen dimensions (avail): { screen.availWidth } x { screen.availHeight } </p>
          <p> Device pixel ratio: { window.devicePixelRatio } </p>
          <p> Root Element Dimensions: { root.clientWidth } x { root.clientHeight } </p>
          <p> Window Inner Dimensions: { window.innerWidth } x { window.innerHeight } </p>
        </Col>
      </Row>
    </Container>
  }
}