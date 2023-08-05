"use client"
import BufferedFormControl from "@/app/components/BufferedFormControl";
import { withConfirm } from "@/app/components/Confirm";
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { Award, AwardRecipient } from "@/app/ws-schema";
import { faInfoCircle, faTimes, faTrash } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Spec } from "immutability-helper";
import React, { useEffect, useState } from "react";
import { Button, Card, Col, InputGroup, ListGroup, Row } from "react-bootstrap";
import { v4 as uuid } from 'uuid';
import update from "immutability-helper";
import { nullIfEmpty } from "@/app/support/strings";

export default withPermission(["ManageAwards"], function EventWizardAwards() {
  let [ awards, setAwards ] = useState<Award[]>([]);
  
  const { subscribe, unsubscribe, call } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    let cb = [
      subscribe<"awards/awards">("awards/awards", setAwards)
    ];
    return () => unsubscribe(cb);
  }, []);
  
  return <React.Fragment>
    <h3> Manage Awards </h3>
    <p className="text-muted"> <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; FINALIST and WINNER awards will automatically be generated, and do not need to be populated in here. </p>

    <Row>
      <Col>
        <InputGroup>
          <InputGroup.Text>New Award Name</InputGroup.Text>
          <BufferedFormControl
            value={""}
            resetOnUpdate
            onEnter={v => call<"awards/set_award">("awards/set_award", { award: { id: uuid(), name: v as string, recipients: [] } }).catch(addError)}
          />
        </InputGroup>
      </Col>
    </Row>

    <Row className="mt-4 flex-wrap">
      {
        awards.map((award, i) => {
          return <Col style={{ minWidth: '400px', maxWidth: '500px' }}>
            <AwardCard
              award={award}
              onDelete={() => call<"awards/delete_award">("awards/delete_award", { award_id: award.id }).catch(addError)}
              onUpdate={(spec) => call<"awards/set_award">("awards/set_award", { award: update(award, spec) }).then(award => setAwards(update(awards, { [i]: { $set: award } }))).catch(addError)}
            />
          </Col>;
        })
      }
    </Row>
  </React.Fragment>
})

function AwardCard({ award, onDelete, onUpdate }: { award: Award, onDelete: () => void, onUpdate: (update: Spec<Award>) => void }) {
  const [ newDetails, setNewDetails ] = useState<AwardRecipient>({ team: null, awardee: null });
  
  return <Card>
    <Card.Header>
      <Row>
        <Col>
          {award.name}
        </Col>
        <Col md="auto">
          <Button variant="danger" size="sm" onClick={() => withConfirm(onDelete)}>
            <FontAwesomeIcon icon={faTrash} />
          </Button>
        </Col>
      </Row>
    </Card.Header>
    <Card.Body>
      <ListGroup>
        {award.recipients.map((recipient, i) => <ListGroup.Item className="py-1">
          <Row>
            <Col>
              {[recipient.team, recipient.awardee].filter(x => x).join(" - ")}
            </Col>
            <Col md="auto">
              <a className="text-light mx-2" onClick={() => withConfirm(() => onUpdate({ recipients: { $splice: [[i, 1]] } }))}>
                <FontAwesomeIcon icon={faTimes} size="sm" />
              </a>
            </Col>
          </Row>
        </ListGroup.Item>)}
        <ListGroup.Item>
          <Row>
            <Col>
              <BufferedFormControl
                auto
                size="sm"
                value={newDetails.team || ""}
                onUpdate={v => setNewDetails({ ...newDetails, team: nullIfEmpty(v as string) })}
                placeholder="Team..."
                onEnter={() => onUpdate({ recipients: { $push: [newDetails] } })}
              />
            </Col>
            <Col>
              <BufferedFormControl
                auto
                size="sm"
                value={newDetails.awardee || ""}
                onUpdate={v => setNewDetails({ ...newDetails, awardee: nullIfEmpty(v as string) })}
                placeholder="Awardee..."
                onEnter={() => onUpdate({ recipients: { $push: [newDetails] } })}
              />
            </Col>
          </Row>
        </ListGroup.Item>
      </ListGroup>
    </Card.Body>
  </Card>
}