import { Spec } from "immutability-helper";
import React from "react";
import { Button, Card, Col, Row } from "react-bootstrap";
import { withValU } from "support/util";
import { WebsocketComponent } from "support/ws-component";
import { SerializedMatch, SupportTicket, TicketComment } from "ws-schema";
import { get_csa_name } from "./CSAIndex";
import update from "immutability-helper";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCheck, faChevronCircleLeft, faTimes, faUserCheck, faUserSlash } from "@fortawesome/free-solid-svg-icons";
import moment from "moment";
import BufferedFormControl from "components/elements/BufferedFormControl";
import { nullIfEmpty } from "support/strings";
import { Link } from "react-router-dom";
import MatchLogView from "monitor/MatchLogView";

type TicketProps = {
  id: number,
  fta: boolean
};

type TicketState = {
  ticket?: SupportTicket,
  match?: SerializedMatch,
}

export default class TicketView extends WebsocketComponent<TicketProps, TicketState> {
  readonly state: TicketState = {};
  
  componentDidMount = () => {
    this.handles = [
      this.listenFn<SupportTicket[]>("Ticket/All", ticks => this.setState({ ticket: ticks.find(t => t.id === this.props.id) })),
      this.listenFn<SerializedMatch[]>("Match/All", matches => this.setState(state => ({ match: matches.find(m => m.id === state.ticket?.match_id) })))
    ]
  }

  update = (spec: Spec<SupportTicket>) => {
    this.send({
      Ticket: {
        Insert: update(this.state.ticket!, spec)
      }
    })
  }

  renderTicket = (ticket: SupportTicket) => {
    const { match } = this.state;
    const my_name = get_csa_name(this.props.fta) || "Unnamed CSA";
    const is_mine = ticket.assigned_to === my_name;

    return <React.Fragment>
      <Row className="ticket-header">
        <Col>
          <h4>
            <Link to="../">
              <Button
                variant="secondary"
              > <FontAwesomeIcon icon={faChevronCircleLeft} /> </Button>
            </Link> &nbsp;
            Team {ticket.team} - { ticket.issue_type }
            { withValU(match, m => ` in ${m.name}`) }
            { ticket.resolved ? <span className="text-success">
              &nbsp; [RESOLVED]
            </span> : undefined }
          </h4>
          <p className="text-muted">
            { withValU(ticket.assigned_to, a => `Assigned to: ${a}`) || "Awaiting Assignment" }
          </p>
        </Col>
        <Col className="text-end" md="auto">
          {
            is_mine ? (
              <Button className="mx-2"
                variant={ticket.resolved ? "danger" : "success"}
                onClick={() => this.update({ resolved: { $set: !ticket.resolved } })}
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
                onClick={() => this.update({ assigned_to: { $set: null } })}
                disabled={ticket.resolved}
              >
                <FontAwesomeIcon icon={faUserSlash} />
               &nbsp; Unassign Me
              </Button>
              : <Button
                variant="success"
                onClick={() => this.update({ assigned_to: { $set: my_name } })}
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
            resetOnEnter
            size="sm"
            placeholder="New Comment"
            value={""}
            disabled={ticket.resolved}
            onEnter={(v) => nullIfEmpty(String(v)) && this.update({
              notes: { $push: [{
                author: my_name,
                comment: v,
                time: moment().utc().unix()
              } as TicketComment]}
            })}
          />
          <br />
          {
            ticket.notes.slice(0).reverse().map(note => <Card className="ticket-note">
              <Card.Body>
                <p className="m-0"> { note.comment } </p>
                <p className="ticket-note-time text-muted"> { note.author } @ { moment.unix(note.time).calendar() } </p>
              </Card.Body>
            </Card>)
          }
        </Col>
        <Col md={9}>
          {
            ticket.match_id ? <MatchLogView autoload team={ticket.team} match_id={ticket.match_id} /> :
              <h6 className="text-muted"> No Logs Available - No Associated Match </h6>
          }
          
        </Col>
      </Row>
    </React.Fragment>
  }

  render() {
    return <Col className="col-full">
      { this.state.ticket ? this.renderTicket(this.state.ticket) : <h3> Loading... </h3> }
    </Col>
  }
}
