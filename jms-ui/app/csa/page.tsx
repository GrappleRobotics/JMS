"use client"
import React, { useEffect, useState } from "react";
import { withPermission } from "../support/permissions";
import { useWebsocket } from "../support/ws-component";
import { Button, Col, ListGroup, Row } from "react-bootstrap";
import { Match, SupportTicket, Team } from "../ws-schema";
import { useErrors } from "../support/errors";
import { useRouter } from "next/navigation";
import Link from "next/link";

export default withPermission(["Ticketing"], function CSAView() {
  const { user, call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  const [ teams, setTeams ] = useState<Team[]>([]);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ tickets, setTickets ] = useState<SupportTicket[]>([]);

  const refreshTickets = () => {
    call<"tickets/all">("tickets/all", null).then(setTickets).catch(addError);
  }

  useEffect(() => {
    let cbs = [
      subscribe<"team/teams">("team/teams", setTeams),
      subscribe<"matches/matches">("matches/matches", setMatches),
    ];
    refreshTickets();
    return () => unsubscribe(cbs);
  }, []);

  return <React.Fragment>
    <Row className="mt-3">
      <Col>
        <h3> Hello { user?.realname } </h3>
      </Col>
      <Col md="auto">
        <Button variant="success" onClick={() => refreshTickets()}>
          Refresh
        </Button> &nbsp;
        <Button variant="orange">
          New Ticket
        </Button>
      </Col>
    </Row>
    <Row className="flex-wrap">
      <Col>
        <Row>
          <Col>
            <h4> My Tickets </h4>
            <ListGroup>
              { tickets.filter(t => t.resolved === false && t.assigned_to === user?.username).map(t => <TicketComponent key={t.id} ticket={t} matches={matches} teams={teams} />) }
            </ListGroup>
          </Col>
        </Row>
        <Row className="mt-2">
          <Col>
            <h4> Unassigned Tickets </h4>
            <ListGroup>
              { tickets.filter(t => t.resolved === false && t.assigned_to === null).map(t => <TicketComponent key={t.id} ticket={t} matches={matches} teams={teams} />) }
            </ListGroup>
          </Col>
        </Row>
      </Col>
      <Col>
        <h4> Resolved Tickets </h4>
        <ListGroup>
          { tickets.filter(t => t.resolved === true).map(t => <TicketComponent key={t.id} ticket={t} matches={matches} teams={teams} />) }
        </ListGroup>
      </Col>
    </Row>
  </React.Fragment>
});

function TicketComponent({ ticket, matches, teams }: { ticket: SupportTicket, matches: Match[], teams: Team[] }) {
  return <Link href={`/csa/${ticket.id}`}>
    <ListGroup.Item
      action
      className="support-ticket-preview"
      data-resolved={ ticket.resolved }
    >
      <Row>
        <Col md={2}> { ticket.team } </Col>
        <Col md={4}> { ticket.issue_type } </Col>
        <Col md={3} className="text-muted"> { ticket.assigned_to || "Unassigned" } </Col>
        <Col md={3}> { ticket.match_id && matches.find(m => m.id === ticket.match_id)?.name } </Col>
      </Row>
    </ListGroup.Item>
  </Link>
}