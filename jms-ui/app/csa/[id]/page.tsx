"use client"

import BufferedFormControl from "@/app/components/BufferedFormControl";
import MatchLogView from "@/app/match-logs/view";
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions"
import { nullIfEmpty } from "@/app/support/strings";
import { useWebsocket } from "@/app/support/ws-component";
import { Match, MatchLog, SupportTicket, Team } from "@/app/ws-schema";
import { faCheck, faChevronCircleLeft, faTimes, faUserCheck, faUserSlash } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import moment from "moment";
import Link from "next/link";
import React, { useEffect, useState } from "react";
import { Button, Card, Col, Row } from "react-bootstrap";

export default withPermission(["Ticketing"], function CSATicket({ params }: { params: { id: string } }) {
  const { user, call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  const [ ticket, setTicket ] = useState<SupportTicket>();
  const [ teams, setTeams ] = useState<Team[]>([]);
  const [ matches, setMatches ] = useState<Match[]>([]);

  const [ matchLog, setMatchLog ] = useState<MatchLog>();

  const refreshTicket = () => {
    let cbs = [
      subscribe<"team/teams">("team/teams", setTeams),
      subscribe<"matches/matches">("matches/matches", setMatches),
    ];
    call<"tickets/get">("tickets/get", { id: params.id }).then(t => {
      setTicket(t);
      if (t.match_id) {
        call<"tickets/get_match_log">("tickets/get_match_log", { match_id: t.match_id, team: t.team })
          .then(setMatchLog)
          .catch((e) => setMatchLog(undefined));
      }
    }).catch(addError);
    return () => unsubscribe(cbs);
  }

  useEffect(() => {
    refreshTicket();
  }, []);

  if (!ticket)
    return <h3> Loading ... </h3>;

  const is_mine = ticket.assigned_to === user?.username;
  const team = teams.find(t => t.number === ticket.team);
  const match = matches.find(m => m.id === ticket.match_id);

  return <React.Fragment>
    <Row className="ticket-header mt-3">
      <Col>
        <h4>
          <Link href="/csa">
            <Button
              variant="secondary"
            > <FontAwesomeIcon icon={faChevronCircleLeft} /> </Button>
          </Link> &nbsp;
          Team { team?.display_number || ticket.team } - { ticket?.issue_type }
          { match && ` in ${match?.name}` }
          { ticket.resolved ? <span className="text-success">
            &nbsp; [RESOLVED]
          </span> : undefined }
        </h4>
        <p className="text-muted">
          { ticket.assigned_to ? `Assigned to ${ticket.assigned_to}` : `Awaiting Assignment` }
        </p>
      </Col>
      <Col className="text-end" md="auto">
        {
          is_mine ? (
            <Button className="mx-2"
              variant={ticket.resolved ? "danger" : "success"}
              onClick={() => call<"tickets/resolve">("tickets/resolve", { id: ticket.id, resolve: !ticket.resolved }).then(setTicket).catch(addError)}
            >
              <FontAwesomeIcon icon={ticket.resolved ? faTimes : faCheck} />
              &nbsp; { ticket.resolved ? "Unresolve" : "Mark Resolved" }
            </Button>
            ) : undefined
        }

        {
          is_mine ? 
            <Button
              variant="danger"
              onClick={() => call<"tickets/assign">("tickets/assign", { id: ticket.id, assign: false }).then(setTicket).catch(addError)}
              disabled={ticket.resolved}
            >
              <FontAwesomeIcon icon={faUserSlash} />
              &nbsp; Unassign Me
            </Button>
            : <Button
              variant="success"
              onClick={() => call<"tickets/assign">("tickets/assign", { id: ticket.id, assign: true }).then(setTicket).catch(addError)}
              disabled={ticket.assigned_to != null}
            >
              <FontAwesomeIcon icon={faUserCheck} />
              &nbsp; Assign to Me 
            </Button>
        }
      </Col>
    </Row>
    <Row className="mb-3 ticket-content">
      <Col md={3} className="ticket-notes">
        <h4> Notes </h4>
        <BufferedFormControl
          resetOnUpdate
          size="sm"
          placeholder="New Comment"
          value={""}
          disabled={ticket.resolved}
          onEnter={(v) => nullIfEmpty(String(v)) && call<"tickets/push_comment">("tickets/push_comment", { id: ticket.id, comment: v as string }).then(setTicket).catch(addError)}
        />
        <br />
        {
          ticket.notes.slice(0).reverse().map((note, i) => <Card className="ticket-note my-2" key={i}>
            <Card.Body>
              <p className="m-0"> { note.comment } </p>
              <p className="ticket-note-time text-muted my-0"> { note.author } @ { moment(note.time).calendar() } </p>
            </Card.Body>
          </Card>)
        }
      </Col>
      <Col md={9}>
        {
          matchLog ? <MatchLogView matchLog={matchLog} /> : <h3>No Match Log Available</h3>
        }
      </Col>
    </Row>
  </React.Fragment>
});