import { faInfoCircle, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BufferedFormControl from "components/elements/BufferedFormControl";
import { Button, Col, Form, Row } from "react-bootstrap";
import { nullIfEmpty } from "support/strings";
import { WebsocketComponent } from "support/ws-component";
import { EventDetails } from "ws-schema";
import { EventWizardPageContent } from "./EventWizard";

type ConfigureEventState = {
  details: EventDetails
}

export default class ConfigureEvent extends WebsocketComponent<{ }, ConfigureEventState> {
  readonly state: ConfigureEventState = { details: { webcasts: [] } };

  componentDidMount = () => this.handles = [
    this.listen("Event/Details/Current", "details")
  ]

  changeEventDetails = (changes: Partial<EventDetails>) => {
    let details = this.state.details;
    this.send({ Event: { Details: { Update: { ...details, ...changes } } } });
  }

  submitNewWebcast = (webcast: string) => {
    this.changeEventDetails({
      webcasts: [ ...this.state.details.webcasts, webcast ]
    });
  }

  deleteWebcast = (i: number) => {
    let webcasts = this.state.details.webcasts;
    webcasts.splice(i, 1);
    this.changeEventDetails({
      webcasts: webcasts
    });
  }

  render() {
    let { details } = this.state;
    return <EventWizardPageContent tabLabel="Configure Event Details" attention={!this.state.details?.event_name}>
      <h4> Configure Event Details </h4>
      <br />

      <Form>
        <Row>
          <Col md={4}>
            <Form.Label>Event Code <span className="text-muted">(Optional)</span></Form.Label>
            <BufferedFormControl 
              type="text"
              placeholder="2022myevent"
              value={details.code || ""}
              onUpdate={v => this.changeEventDetails({ code: nullIfEmpty(String(v)) })}
            />
            <Form.Text className="text-muted">
              <FontAwesomeIcon icon={faInfoCircle} /> The event code is usually provided by <i>FIRST</i>
            </Form.Text>
          </Col>
          <Col>
            <Form.Label>Event Name</Form.Label>
            <BufferedFormControl
              type="text"
              placeholder="My Really Groovy Robotics Event"
              value={details.event_name || ""}
              onUpdate={v => this.changeEventDetails({ event_name: nullIfEmpty(String(v)) })}
            />
          </Col>
        </Row>
        <Row className="my-3">
          <Col>
            <Form.Label> Webcast URLs (TBA) <span className="text-muted">(Optional)</span> </Form.Label>
            <BufferedFormControl
              type="text"
              placeholder="New Webcast URL, e.g. https://www.youtube.com/watch?v=dQw4w9WgXcQ"
              value=""
              onUpdate={ v => this.submitNewWebcast(String(v)) }
              resetOnEnter
            />
            {
              details.webcasts?.map((wc, i) => <Row key={i} className="my-2">
                <Col md="auto">
                  <Button
                    size="sm"
                    variant="danger"
                    onClick={() => this.deleteWebcast(i)}
                  >
                    <FontAwesomeIcon icon={faTimes} /> &nbsp; Delete
                  </Button>
                </Col>
                <Col>
                  <a href={wc} target="_blank">{ wc }</a>
                </Col>
              </Row>)
            }
          </Col>
        </Row>
      </Form>
    </EventWizardPageContent>
  }
}