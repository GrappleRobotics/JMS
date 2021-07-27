import { faCalculator, faFlagCheckered, faMagic, faRobot } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { EVENT_WIZARD, MATCH_CONTROL, REFEREE, SCORING } from "paths";
import React from "react";
import { Container } from "react-bootstrap";

export default class Home extends React.PureComponent {
  render() {
    return <Container>
      <h2> Welcome to JMS! </h2>
      <p> Click one of the links below to get started. </p>
      <br />
      <ul>
        <li>
          <h4> Scoring Table </h4>
          <ul>
            <li className="h5"> <a href={ EVENT_WIZARD }> <FontAwesomeIcon icon={faMagic} /> &nbsp; Event Wizard </a> </li>
            <li className="h5"> <a href={ MATCH_CONTROL }> <FontAwesomeIcon icon={faRobot} /> &nbsp; Match Control </a> </li>
          </ul>
        </li>
        <br />
        <li>
          <h4> Referees and Scorers </h4>
          <ul>
            <li className="h5"> <a href={ SCORING }> <FontAwesomeIcon icon={faCalculator} /> &nbsp; Scorer  </a> </li>
            <li className="h5"> <a href={ REFEREE }> <FontAwesomeIcon icon={faFlagCheckered} /> &nbsp; Referee  </a> </li>
          </ul>
        </li>
      </ul>
    </Container>
  }
}