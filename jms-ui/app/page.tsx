"use client"
import "./index.scss";
import { Button, Card, Col, Row } from "react-bootstrap";
import UserPage from "./userpage";
import Link from "next/link";
import { PermissionGate } from "./support/permissions";

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
  return (
    <UserPage container>
      <h2 className="my-4"> Welcome to JMS! </h2>

      <HomeTileSep> At the Scoring Table </HomeTileSep>
      <HomeTileRow>
        <HomeTile name="Event Wizard" href="/wizard" img="wizard.jpg"> Configure your event, generate schedules, and give out awards. </HomeTile>
        <PermissionGate permissions={["FTA", "FTAA"]}>
          <HomeTile name="FTA" href="/field-control/fta"> Monitor teams, run matches. </HomeTile>
        </PermissionGate>
      </HomeTileRow>
      <HomeTileSep> On the Field </HomeTileSep>
      <HomeTileSep> In the Stands </HomeTileSep>
    </UserPage>
  )
}
