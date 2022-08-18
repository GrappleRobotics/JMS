import BufferedFormControl from "components/elements/BufferedFormControl";
import { confirmModal } from "components/elements/Confirm";
import React from "react"
import { Button, Col, Container, ListGroup, Row } from "react-bootstrap";
import { Link, Route, Routes, useParams } from "react-router-dom";
import { withValU } from "support/util";
import { WebsocketComponent } from "support/ws-component"
import { SerializedMatch, SupportTicket, Team } from "ws-schema";
import newTicket from "./NewTicket";
import TicketView from "./Ticket";

export function get_csa_name(fta: boolean) {
  return fta ? "FTA" : sessionStorage.getItem("csa_name");
}

export function set_csa_name(name: string) {
  sessionStorage.setItem("csa_name", name);
}

type CSAIndexProps = {
  fta: boolean
};

type CSAIndexState = {
  tickets: SupportTicket[],
  matches: SerializedMatch[],
  teams: Team[]
};

export default class CSAIndex extends WebsocketComponent<CSAIndexProps, CSAIndexState> {
  readonly state: CSAIndexState = { tickets: [], matches: [], teams: []}
  
  componentDidMount = () => {
    if (get_csa_name(this.props.fta) == null) {
      this.promptName();
    }

    this.handles = [
      this.listen("Ticket/All", "tickets"),
      this.listen("Match/All", "matches"),
      this.listen("Event/Team/CurrentAll", "teams")
    ];
  }

  promptName = () => {
    confirmModal("", {
      data: "",
      title: "Set Name",
      okText: "Set Name",
      renderInner: (name: string, onUpdate, ok) => <React.Fragment>
        <p> My name is... </p>
        <BufferedFormControl
          instant
          autofocus
          type="text"
          value={name}
          onUpdate={v => onUpdate(String(v))}
          onEnter={ok}
        />
      </React.Fragment>
    }).then(set_csa_name)
  }

  renderTickets = (tickets: SupportTicket[]) => {
    if (tickets.length === 0)
      return <p className="text-muted"> No tickets to display </p>;
    
    const { matches } = this.state;

    return <ListGroup>
      {
        tickets.map(t => (
          <Link to={`${t.id}`}>
            <ListGroup.Item
              key={t.id} action
              className="support-ticket-preview"
              data-resolved={ t.resolved }
            >
              <Row>
                <Col md={2}> { t.team } </Col>
                <Col md={4}> { t.issue_type } </Col>
                <Col md={4} className="text-muted"> { t.assigned_to || "Unassigned" } </Col>
                <Col md={2}> { withValU(t.match_id, id => matches.find(m => m.id === id)?.short_name) } </Col>
              </Row>
            </ListGroup.Item>
          </Link>

        ))
      }
    </ListGroup>
  }

  render() {
    const { fta } = this.props;
    const { tickets, teams, matches } = this.state;
    const my_name = get_csa_name(fta) || "Unnamed CSA";

    return <Container className="csa-index">
      <Row className="mb-3">
        <Col>
          <h3> Hello { my_name } </h3>
        </Col>
        <Col className="text-end">
          <Button
            onClick={this.promptName}
          > Change Name </Button>
          <Button
            className="mx-2"
            onClick={() => newTicket(my_name, teams.map(x => x.id), matches, t => this.send({
              Ticket: { Insert: t }
            }))}
          > New Ticket </Button>
        </Col>
      </Row>
      <Row className="flex-wrap">
        <Col>
          <Row>
            <Col>
              <h4> My Tickets </h4>
              {
                this.renderTickets(tickets.filter(t => my_name != null && t.assigned_to?.toLowerCase() === my_name.toLowerCase()))
              }
            </Col>
          </Row>
          <Row className="mt-2">
            <Col>
              <h4> Unassigned Tickets </h4>
              { this.renderTickets(tickets.filter(t => t.assigned_to === null)) }
            </Col>
          </Row>
        </Col>
        <Col>
          <h4> Other Tickets </h4>
          { this.renderTickets(tickets.filter(t => my_name === null || (t.assigned_to != null && t.assigned_to?.toLowerCase() != my_name.toLowerCase()))) }
        </Col>
      </Row>
    </Container>
  }
}

function TicketViewWrapper(props: { fta: boolean }) {
  const { id } = useParams();
  if (id != null)
    return <TicketView id={parseInt(id)} fta={props.fta} />;
  return <h4> Page not found! </h4>
}

export function CSARouter(props: { fta: boolean }) {
  return <Routes>
    <Route path="/" element={ <CSAIndex fta={props.fta} /> } />
    <Route path="/:id" element={ <TicketViewWrapper fta={props.fta} /> } />
  </Routes>
}