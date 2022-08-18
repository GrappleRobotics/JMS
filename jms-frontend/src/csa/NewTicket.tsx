import BufferedFormControl from "components/elements/BufferedFormControl";
import { confirmModal } from "components/elements/Confirm";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import moment from "moment";
import React from "react";
import { Form } from "react-bootstrap";
import { SerializedMatch, SupportTicket } from "ws-schema";
import update from "immutability-helper";
import { nullIfEmpty } from "support/strings";

const ISSUE_TYPES = [
  "CODE",
  "ROBORIO",
  "RADIO",
  "LAPTOP",
  "POWER",
  "CAN",
  "E-STOP",
  "E-STOP (FTA)",
  "UNSAFE",
  "OTHER"
];

export default function newTicket(csa_name: string, team: number | number[], match: SerializedMatch | SerializedMatch[], on_ticket: (ticket: SupportTicket) => void) {
  const match_picker = Array.isArray(match);
  const team_picker = Array.isArray(team);

  confirmModal("", {
    data: {
      team: team_picker ? team[0] : team,
      match_id: match_picker ? undefined : match?.id,
      issue_type: "OTHER",
      author: csa_name,
      notes: [{
        author: csa_name,
        time: 0,
        comment: ""
      }],
      resolved: false
    },
    size: "lg",
    title: `Create Ticket`,
    okText: "Submit Ticket",
    renderInner: (ticket: SupportTicket, onUpdate) => <React.Fragment>
      {
        team_picker ? <Form.Select className="my-2" value={ticket.team || team[0]} onChange={t => onUpdate(update(ticket, { team: { $set: Number(t.target.value) } }))}>
          {
            team.map(t => <option key={t} value={t}>Team {t}</option>)
          }
        </Form.Select> : undefined
      }

      {
        match_picker ? <Form.Select className="my-2" value={ticket.match_id || ""} onChange={id => onUpdate(update(ticket, { match_id: { $set: nullIfEmpty(id.target.value) } }))}>
          <option value=""> No Match </option>
          {
            match.map(m => <option key={m.id} value={m.id || ""}> { m.name } </option>)
          }
        </Form.Select> : undefined
      }

      <EnumToggleGroup
        className="flex-wrap"
        name="issue_group"
        values={ISSUE_TYPES}
        value={ticket.issue_type}
        onChange={(v) => onUpdate(update(ticket, { issue_type: { $set: v } }))}
        variant="outline-warning"
        variantActive="danger"
      />
      <br /> <br />
      <Form.Label> Notes </Form.Label>
      <BufferedFormControl
        instant
        autofocus
        as="textarea"
        placeholder="Team lost battery on field..."
        value={ticket.notes[0].comment}
        onUpdate={(v) => onUpdate(update(ticket, { notes: { 0: { comment: { $set: String(v) } } } }))}
      />
    </React.Fragment>
  }).then(ticket => {
    ticket.notes[0].time = moment().utc().unix();
    on_ticket(ticket)
    // this.props.new_ticket(ticket)
  })
}