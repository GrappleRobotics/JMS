"use client";
import "./wizard.scss";
import React from "react";
import UserPage from "../userpage";
import { Card, Col, Nav, Row } from "react-bootstrap";
import { usePathname, useRouter } from "next/navigation";

function WizardTabLink({ link, children }: { link: string, children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();

  return <Nav.Item>
    <Nav.Link className="event-wizard-tab-link my-1" data-active={pathname.startsWith(`/wizard/${link}`)} onClick={() => router.push(`/wizard/${link}`)}>
      { children }
    </Nav.Link>
  </Nav.Item>
}

export default function WizardLayout({ children }: { children: React.ReactNode }) {
  return <UserPage container>
    <br />
    <Row>
      <Col className="event-wizard-nav" md={3}>
        <Nav variant="pills" className="flex-column">
          <br /> <h6 className="text-muted"> User Management </h6>
          <WizardTabLink link="users">Manage Users</WizardTabLink>
          <br /> <h6 className="text-muted"> Pre-Event </h6>
          <WizardTabLink link="event">Event Details</WizardTabLink>
          <WizardTabLink link="teams">Manage Teams</WizardTabLink>
          <WizardTabLink link="schedule">Manage Schedule</WizardTabLink>
          <WizardTabLink link="quals">Qualification Schedule</WizardTabLink>
          <br /> <h6 className="text-muted"> Playoffs </h6>
          <WizardTabLink link="alliances">Alliance Selections</WizardTabLink>
          <WizardTabLink link="playoffs">Playoff Schedule</WizardTabLink>
          <br /> <h6 className="text-muted"> Awards </h6>
          <WizardTabLink link="awards">Manage Awards</WizardTabLink>
          <br /> <h6 className="text-muted"> Advanced </h6>
          <WizardTabLink link="advanced-networking">Advanced Networking</WizardTabLink>
          <WizardTabLink link="electronics">Field Electronics</WizardTabLink>
          <WizardTabLink link="backup">Backups</WizardTabLink>
          <br /> <h6 className="text-muted"> Integrations </h6>
          <WizardTabLink link="integrations/tba">The Blue Alliance</WizardTabLink>
        </Nav>
      </Col>
      <Col className="event-wizard-view">
        <Card>
          <Card.Body>
            { children }
          </Card.Body>
        </Card>
      </Col>
    </Row>
  </UserPage>
}