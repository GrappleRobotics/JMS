import { Spec } from "immutability-helper";
import React from "react";
import { Button, Card, Col, Container, Row } from "react-bootstrap";
import { withValU } from "support/util";
import { WebsocketComponent } from "support/ws-component";
import { MatchStationStatusRecord, MatchStationStatusRecordKey, SerializedMatch, SupportTicket, TicketComment, TicketMessageLogs2UI } from "ws-schema";
import { get_csa_name } from "./CSAIndex";
import update from "immutability-helper";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCheck, faChevronCircleLeft, faCircleNotch, faDownload, faTimes, faTriangleExclamation, faUserCheck, faUserSlash } from "@fortawesome/free-solid-svg-icons";
import moment from "moment";
import BufferedFormControl from "components/elements/BufferedFormControl";
import { nullIfEmpty } from "support/strings";
import { Link } from "react-router-dom";

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
          <h3>
            <Link to="../">
              <Button
                variant="secondary"
              > <FontAwesomeIcon icon={faChevronCircleLeft} /> </Button>
            </Link> &nbsp;
            Team {ticket.team} - { ticket.issue_type }
            { withValU(match, m => ` in ${m.name}`) }
            { ticket.resolved ? <span className="text-success">
              &nbsp; [<FontAwesomeIcon icon={faCheck} />&nbsp;RESOLVED]
            </span> : undefined }
          </h3>
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
        <Col>
          <TicketLogView ticket={ticket} />
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

type TicketLogViewState = {
  data?: MatchStationStatusRecord,
  loading: boolean,
  error?: string
}

class TicketLogView extends WebsocketComponent<{ ticket: SupportTicket }, TicketLogViewState> {
  readonly state: TicketLogViewState = { loading: false };
 
  loadLogs = () => {
    const key: MatchStationStatusRecordKey = {
      team: this.props.ticket.team,
      match_id: this.props.ticket.match_id!
    };
    const localStorageKey = JSON.stringify({ type: "matchLog", key: key });

    const cached = localStorage.getItem(localStorageKey);
    if (cached != null) {
      this.setState({ data: JSON.parse(cached) });
    } else {
      this.setState({ loading: true }, () => (
        this.transact<MatchStationStatusRecord | null>({
          Ticket: { Logs: { Load: key } }
        }, "Ticket/Logs/Load")
        .then(data => {
          if (data.msg != null) {
            localStorage.setItem(localStorageKey, JSON.stringify(data.msg));
            this.setState({ loading: false, data: data.msg });
          } else {
            this.setState({ loading: false, error: "No Record Exists" });
          }
        }).catch((reason) => this.setState({ loading: false, error: reason }))
      ));
    }
  }
  
  render() {
    const { data, loading, error } = this.state;
    return error ? (
      <h4 className="text-warning">
        <FontAwesomeIcon icon={faTriangleExclamation} />
        &nbsp; Could not load record: { error }
      </h4>)
      : loading || data == null ? (
        <Button size="lg" onClick={this.loadLogs} disabled={loading}>
          <FontAwesomeIcon icon={loading ? faCircleNotch : faDownload} spin={loading} />
          &nbsp;&nbsp;Load Match Logs
        </Button>
      ) : (
        <React.Fragment>

        </React.Fragment> 
      )

  }
}