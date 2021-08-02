import { faExclamationTriangle, faInfoCircle, faPlus, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Button, Card, Col, Form, ListGroup, Row } from "react-bootstrap";
import { nullIfEmpty } from "support/strings";
import { confirm } from "react-bootstrap-confirmation";
import EditableFormControl from "components/elements/EditableFormControl";

class AwardCard extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      newTeam: null,
      newAwardee: null
    }
  }

  delete = async () => {
    let award = this.props.award;

    if (award.recipients.length > 0) {
      let result = await confirm("Are you sure?", {
        title: `Delete "${award.name}"?`,
        okButtonStyle: "success"
      });

      if (!result)
        return;
    }

    this.props.ws.send("event", "awards", "delete", award.id);
  }

  addRecipient = () => {
    if (this.state.newTeam || this.state.newAwardee) {
      let recipients = [...this.props.award.recipients, {
        team: this.state.newTeam ? parseInt(this.state.newTeam) : null,
        awardee: this.state.newAwardee
      }];

      this.props.ws.send("event", "awards", "update", {
        ...this.props.award,
        recipients
      });

      this.setState({ newTeam: null, newAwardee: null });
    }
  }

  deleteRecipient = async (idx) => {
    let result = await confirm("Are you sure?", {
      title: `Delete Award Recipient?`,
      okButtonStyle: "success"
    });

    if (result)
      this.props.ws.send("event", "awards", "update", {
        ...this.props.award,
        recipients: this.props.award.recipients.filter((v, i) => i !== idx)
      });
  }

  updateName = (name) => {
    if (!!nullIfEmpty(name)) {
      this.props.ws.send("event", "awards", "update", {
        ...this.props.award,
        name: name
      });
    }
  }
  
  render() {
    let award = this.props.award;
    return <Card className="award">
      <Card.Header>
        <Row>
          <Col md={10}>
            <h5 className="my-0">
              <EditableFormControl
                autofocus
                type="text"
                value={ award.name }
                onUpdate={ this.updateName }
              />
            </h5>
          </Col>
          <Col md={2}>
            <a className="text-danger mx-2" onClick={this.delete}> 
              <FontAwesomeIcon icon={faTimes} size="lg" />
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
                  onChange={ e => this.setState({ newTeam: nullIfEmpty(e.target.value) }) }
                  onKeyDown={ e => e.key === 'Enter' ? this.addRecipient() : null }
                  className="bg-dark text-light"
                />
              </Col>
              <Col>
                <Form.Control
                  type="text"
                  size="sm"
                  placeholder="Person"
                  value={this.state.newAwardee || ""}
                  onChange={ e => this.setState({ newAwardee: nullIfEmpty(e.target.value) }) }
                  onKeyDown={ e => e.key === 'Enter' ? this.addRecipient() : null }
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

export default class ConfigureAwards extends React.Component {
  static eventKey() { return "configure_awards"; }
  static tabName() { return "Configure Awards"; }

  submitNewAward = v => {
    let val = nullIfEmpty(v);
    if (val) {
      this.props.ws.send("event", "awards", "create", val);
    }
  }

  newAwardKeyDown = e => {
    if (e.key === 'Enter') {
      this.submitNewAward(e.target.value);
      e.target.value = "";
    }
  }

  render() {
    return <div>
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
            onKeyDown={this.newAwardKeyDown}
          />
        </Col>
      </Row>
      <Row className="my-3 flex-wrap">
        {
          this.props.awards?.map(a => <Col className="my-2">
            <AwardCard award={a} ws={this.props.ws} />
          </Col>)
        }
      </Row>
    </div>
  }
}