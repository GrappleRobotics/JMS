import { AUDIENCE, AUDIENCE_CONTROL, DEBUG, ESTOPS, EVENT_WIZARD, MATCH_CONTROL, MONITOR, RANKINGS, RANKINGS_NO_SCROLL, REFEREE, REPORTS, SCORING, TIMER } from "paths";
import React from "react";
import { Card, Col, Container, Row } from "react-bootstrap";
import { Link } from "react-router-dom";
import { withVal } from "support/util";

type HomeTileProps = {
  href: string,
  name: string,
  img?: string,
  children?: string,
}

class HomeTile extends React.PureComponent<HomeTileProps> {
  render() {
    const { href, name, children, img } = this.props;
    return <Col className="home-tile" data-tile-name={name}>
      <Link to={href}>
        <Card>
          { withVal(img, img => <Card.Img className="home-tile-bg-img" src={`/img/tiles/${img}`} />) }
          <Card.ImgOverlay data-tile-name={name}>
            <Card.Title> { name } </Card.Title>
            <Card.Subtitle> {children} </Card.Subtitle>
          </Card.ImgOverlay>
        </Card>
      </Link>
    </Col>
  }
}

const HomeTileSep = (props: { children: React.ReactNode }) => <Row className="home-tile-sep">
  { props.children }
</Row>

const HomeTileRow = (props: React.ComponentProps<typeof Row>) => <Row className="flex-wrap" {...props} />

export default class Home extends React.PureComponent<{ fta: boolean }> {
  render() {
    const fta = this.props.fta;
    
    return <Container className="home">
      <h2> Welcome to JMS! </h2>
      <br />
      <HomeTileSep> From the Scoring Table </HomeTileSep>
      <HomeTileRow>
        <HomeTile name="Event Wizard" href={EVENT_WIZARD} img="wizard.jpg"> Configure your event, generate schedules, and give out awards. </HomeTile>
        <HomeTile name="Match Control" href={MATCH_CONTROL} img="matchcontrol.jpg"> Scorekeeper controls for match flow. </HomeTile>
        <HomeTile name="Field Monitor" href={MONITOR} img="fieldmon.png"> Monitor the status of robots on the field. </HomeTile>
        <HomeTile name="Reports" href={REPORTS} img="reports.jpg"> Generate schedule, team, and advancement reports. </HomeTile>
        <HomeTile name="A/V Controls" href={AUDIENCE_CONTROL} img="audiencecontrol.jpg"> Control the Audience Display from the AV Desk. </HomeTile>
        {
          fta ? <HomeTile name="DEBUG" href={DEBUG}> Debug Information </HomeTile> : <React.Fragment />
        }
      </HomeTileRow>

      <HomeTileSep> On the Field </HomeTileSep>
      <HomeTileRow>
        <HomeTile name="Referee Panels" href={REFEREE} img="referee.jpg"> Referee matches, control the field, and score robot endgames. </HomeTile>
        <HomeTile name="Scorer Panels" href={SCORING} img="scorers.jpg"> Score matches manually when automated scoring is not available. </HomeTile>
        <HomeTile name="Field Timer" href={TIMER} img="timer.png"> Field timer display for teams on the field. </HomeTile>
        <HomeTile name="Emergency Stops" href={ESTOPS} img="estop.png"> Tablet-based emergency stops when physical emergency stop buttons are not available. </HomeTile>
      </HomeTileRow>

      <HomeTileSep> In the Stands </HomeTileSep>
      <HomeTileRow>
        <HomeTile name="Audience Overlay" href={AUDIENCE} img="audience.jpg"> Show match scores, alliance selections, and event messages. </HomeTile>
        <HomeTile name="Rankings" href={RANKINGS} img="rankings.png"> Display team standings in the pits and on the field. </HomeTile>
        <HomeTile name="Rankings (static)" href={RANKINGS_NO_SCROLL} img="rankings.png"> Display team standings in the pits and on the field (no scroll). </HomeTile>
      </HomeTileRow>
    </Container>
  }
}