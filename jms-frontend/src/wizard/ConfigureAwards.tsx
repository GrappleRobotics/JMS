import { faExclamationTriangle, faInfoCircle, faPlus, faTimes, faTrash } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Button, Card, Col, Form, ListGroup, Row } from "react-bootstrap";
import { maybeParseInt, nullIfEmpty, undefinedIfEmpty } from "support/strings";
import EditableFormControl from "components/elements/EditableFormControl";
import { Award } from "ws-schema";
import { WebsocketContext, WebsocketContextT } from "support/ws-component";
import confirmBool from "components/elements/Confirm";
import { EventWizardPageContent } from "./EventWizard";

type AwardCardProps = {
  award: Award,
  onUpdate: (award: Partial<Award>) => void,
  onDelete: (id: number) => void
};

type AwardCardState = {
  newTeam?: number,
  newAwardee?: string
}

class AwardCard extends React.Component<AwardCardProps, AwardCardState> {
  readonly state: AwardCardState = {};

  delete = async () => {
    let award = this.props.award;
    if (award.id == null)
      return;

    if (award.recipients.length > 0) {
      let result = await confirmBool("Are you sure? This award already has recipient(s) assigned", {
        title: `Delete "${award.name}"?`,
        okVariant: "danger",
        okText: "Delete Award"
      });

      if (!result)
        return;
    }

    this.props.onDelete(award.id)
  }

  deleteRecipient = async (idx: number) => {
    let result = await confirmBool("Are you sure?", {
      title: `Delete Award Recipient?`,
      okVariant: "danger",
      okText: "Delete Recipient"
    });

    if (result) {
      this.props.onUpdate({
        recipients: this.props.award.recipients.filter((v, i) => i !== idx)
      });
    }
  }

  addRecipient = () => {
    if (this.state.newTeam || this.state.newAwardee) {
      let recipients = [...this.props.award.recipients, {
        team: this.state.newTeam,
        awardee: this.state.newAwardee
      }];

      this.props.onUpdate({ recipients })

      this.setState({ newTeam: undefined, newAwardee: undefined });
    }
  }
  
  render() {
    const { award, onUpdate } = this.props;

    return <Card className="wizard-award" data-award-name={award.name}>
      <Card.Header>
        <Row>
          <Col md={10}>
            <h5 className="my-0">
              <EditableFormControl
                autofocus
                type="text"
                value={ award.name }
                onUpdate={ name => onUpdate({ name: undefinedIfEmpty(String(name)) }) }
              />
            </h5>
          </Col>
          <Col md={2}>
            <a className="text-danger mx-2" onClick={this.delete}> 
              <FontAwesomeIcon icon={faTrash} size="lg" />
            </a>
          </Col>
        </Row>
      </Card.Header>
      <Card.Body>
        <ListGroup>
          {
            award.recipients.map((recipient, idx) => <ListGroup.Item className="py-1">
              <Row>
                <Col md={10}>
                  {
                    [ recipient.team, recipient.awardee ].filter(x => x).join(" - ")
                  }
                </Col>
                <Col md={2}>
                  <a className="text-light mx-2" onClick={() => this.deleteRecipient(idx)}> 
                    <FontAwesomeIcon icon={faTimes} size="sm" />
                  </a>
                </Col>
              </Row>
            </ListGroup.Item>)
          }
          <ListGroup.Item>
            <Row>
              <Col>
                <Form.Control
                  type="number"
                  size="sm"
                  placeholder="Team"
                  value={this.state.newTeam || ""}
                  onChange={ e => this.setState({ newTeam: maybeParseInt(e.target.value) }) }
                  onKeyDown={ (e: any) => e.key === 'Enter' ? this.addRecipient() : null }
                  className="bg-dark text-light"
                />
              </Col>
              <Col>
                <Form.Control
                  type="text"
                  size="sm"
                  placeholder="Person"
                  value={this.state.newAwardee || ""}
                  onChange={ e => this.setState({ newAwardee: undefinedIfEmpty(e.target.value) }) }
                  onKeyDown={ (e: any) => e.key === 'Enter' ? this.addRecipient() : null }
                  className="bg-dark text-light"
                />
              </Col>
            </Row>
          </ListGroup.Item>
        </ListGroup>
      </Card.Body>
    </Card>
  }
}

type ConfigureAwardsState = {
  awards: Award[]
};

export default class ConfigureAwards extends React.Component {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;
  handles: string[] = [];

  readonly state: ConfigureAwardsState = {
    awards: []
  };

  componentDidMount = () => {
    this.handles = [
      this.context.listen<Award[]>([ "Event", "Award", "CurrentAll" ], msg => this.setState({ awards: msg }))
    ]
  }
  componentWillUnmount = () => this.context.unlisten(this.handles);

  render() {
    return <EventWizardPageContent tabLabel="Assign Awards">
      <h4> Configure Awards </h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, awards are configured. To add a new award, enter the name in the box below and hit 'Enter'.
        <br />
        <FontAwesomeIcon icon={faExclamationTriangle} /> &nbsp;
        <strong> WINNER and FINALIST awards are automatically generated at the conclusion of the playoff matches. </strong>
      </p>
      <br />

      <Row>
        <Col>
          <Form.Control
            type="text"
            placeholder="New Award Name"
            onKeyDown={(e: any) => {
              if (e.key === 'Enter') {
                let val = nullIfEmpty(e.target.value);
                if (val)
                  this.context.send({ Event: { Award: { Create: val } } });
                e.target.value = "";
              }
            }}
          />
        </Col>
      </Row>
      <Row className="my-3 flex-wrap">
        {
          this.state.awards.map(a => <Col className="my-2">
            <AwardCard award={a} onUpdate={award => this.context.send({
              Event: { Award: { Update: { ...a, ...award } } }
            })} onDelete={id => this.context.send({
              Event: { Award: { Delete: id } }
            })} />
          </Col>)
        }
      </Row>
    </EventWizardPageContent>
  }
}