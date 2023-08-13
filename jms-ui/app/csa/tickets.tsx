import React from "react";
import { confirmModal } from "../components/Confirm";
import JmsWebsocket from "../support/ws";
import { SupportTicket } from "../ws-schema";
import EnumToggleGroup from "../components/EnumToggleGroup";
import BufferedFormControl from "../components/BufferedFormControl";
import update from "immutability-helper";
import { InputGroup } from "react-bootstrap";

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
  "COMPLEX",
  "OTHER"
];

export async function newTicketModal(team: number, match_id: string | undefined, call: JmsWebsocket["call"], addError: (e: string) => void) {
  let v = await confirmModal("", {
    data: {
      issue_type: "OTHER",
      comment: ""
    },
    size: "lg",
    title: `Create Ticket`,
    okText: "Submit Ticket",
    renderInner: (ticket, onUpdate) => <React.Fragment>
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
      
      <InputGroup>
        <InputGroup.Text>Comment</InputGroup.Text>
        <BufferedFormControl
          instant
          autofocus
          as="textarea"
          placeholder="Team lost battery on field..."
          value={ticket.comment}
          onUpdate={(v) => onUpdate(update(ticket, { comment: { $set: String(v) } }))}
        />
      </InputGroup>
    </React.Fragment>
  });
  
  call<"tickets/new">("tickets/new", { team: team, issue_type: v.issue_type, match_id: match_id || null })
    .then((ticket) => {
      if (v.comment.trim() != "")
        call<"tickets/push_comment">("tickets/push_comment", { id: ticket.id, comment: v.comment })
          .catch(addError);
    }).catch(addError);
}