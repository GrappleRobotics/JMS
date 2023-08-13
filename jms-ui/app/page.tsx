"use client"
import "./index.scss";
import { Button, Card, Col, Row } from "react-bootstrap";
import UserPage from "./userpage";
import Link from "next/link";
import { PermissionGate } from "./support/permissions";
import { useWebsocket } from "./support/ws-component";

type HomeTileProps = {
  href: string,
  name: string,
  img?: string,
  children?: string,
}

function HomeTile({ href, name, img, children }: HomeTileProps) {
  return <Col className="home-tile" data-tile-name={name}>
    <Link href={href}>
      <Card>
        { img && <Card.Img className="home-tile-bg-img" src={`/img/tiles/${img}`} /> }
        <Card.ImgOverlay data-tile-name={name}>
          <Card.Title> { name } </Card.Title>
          <Card.Subtitle> {children} </Card.Subtitle>
        </Card.ImgOverlay>
      </Card>
    </Link>
  </Col>
}

const HomeTileSep = (props: { children: React.ReactNode }) => <Row className="home-tile-sep">
  { props.children }
</Row>

const HomeTileRow = (props: React.ComponentProps<typeof Row>) => <Row className="flex-wrap" {...props} />

export default function Home() {
  const { user } = useWebsocket();

  return (
    <UserPage container>
      <h2 className="my-4"> Welcome to JMS! </h2>

      <HomeTileSep> At the Scoring Table </HomeTileSep>
      <HomeTileRow>
        { user && <HomeTile name="Event Wizard" href="/wizard" img="wizard.jpg"> Configure your event, generate schedules, and give out awards. </HomeTile> }
        <PermissionGate permissions={["FTA", "FTAA"]}>
          <HomeTile name="FTA" href="/field-control/fta"> Monitor teams, run matches. </HomeTile>
          <HomeTile name="DEBUG" href="/debug"> Debug Controls </HomeTile>
        </PermissionGate>
        <PermissionGate permissions={["EditScores"]}>
          <HomeTile name="Edit Scores" href="/scoring/edit">Edit Scores</HomeTile>
        </PermissionGate>
        <PermissionGate permissions={["ManageAudience"]}>
          <HomeTile name="Audience Display Control" href="/audience/control" img="audiencecontrol.jpg"> Control the Audience Display </HomeTile>
        </PermissionGate>
        <HomeTile name="Reports" href="/reports" img="reports.jpg">Generate Match, Team, and Award Reports</HomeTile>
        <PermissionGate permissions={["Ticketing"]}>
          <HomeTile name="CSA" href="/csa">View match reports, resolve tickets</HomeTile>
        </PermissionGate>
      </HomeTileRow>
      <HomeTileSep> On the Field </HomeTileSep>
      <HomeTileRow>
        <PermissionGate permissions={["Scoring"]}>
          <HomeTile name="Scoring" href="/scoring" img="scorers.jpg"> Score Matches </HomeTile>
          <HomeTile name="Referee" href="/scoring/referee" img="referee.jpg"> Referee Matches, Assign Fouls </HomeTile>
        </PermissionGate>
        <HomeTile name="Match Timer" href="/timer" img="timer.png"> On-Field Match Timers </HomeTile>
        <HomeTile name="Team Estops" href="/estops" img="estop.png"> Team E-Stops </HomeTile>
      </HomeTileRow>
      <HomeTileSep> In the Stands </HomeTileSep>
      <HomeTileRow>
        <HomeTile name="Rankings" href="/rankings" img="rankings.png"> View Team Standings and the Playoff Bracket </HomeTile>
        <HomeTile name="Audience Display" href="/audience" img="audience.jpg"> View the Audience Display </HomeTile>
      </HomeTileRow>
    </UserPage>
  )
}
