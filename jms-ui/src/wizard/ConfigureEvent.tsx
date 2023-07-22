import { faInfoCircle, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BufferedFormControl from "components/elements/BufferedFormControl";
import { Button, Col, Form, Row } from "react-bootstrap";
import { nullIfEmpty } from "support/strings";
import { WebsocketComponent } from "support/ws-component";
import { EventDetails } from "ws-schema";
import { EventWizardPageContent } from "./EventWizard";
import { SketchPicker } from 'react-color';

type ConfigureEventState = {
  details: EventDetails
}

export default class ConfigureEvent extends WebsocketComponent<{ }, ConfigureEventState> {
  readonly state: ConfigureEventState = { details: { webcasts: [], av_event_colour: "#fff", av_chroma_key: "#fff" } };

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
        <br />
        <h4> A/V Setup </h4>
        <Row className="mt-2">
          <Col md="auto">
            <Form.Label> Event Colour <span className="text-muted">(Audience Display)</span> </Form.Label>
            <br />
            <SketchPicker
              disableAlpha={true}
              presetColors={["#e9ab01", "#1f5fb9", "#901fb9"]}
              color={ details.av_event_colour }
              onChangeComplete={ c => this.changeEventDetails({ av_event_colour: c.hex }) }
            />
          </Col>
          <Col md="auto">
            <Form.Label> Chroma Key <span className="text-muted">(Background)</span> </Form.Label>
            <br />
            <SketchPicker
              disableAlpha={true}
              presetColors={["#000", "#f0f", "#0f0", "#333"]}
              color={ details.av_chroma_key }
              onChangeComplete={ c => this.changeEventDetails({ av_chroma_key: c.hex }) }
            />
          </Col>
        </Row>
        <br />
        <p className="text-muted"> 
          <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; 
          If you're using OBS, you can use a "Browser Source" with the following custom CSS to make the window transparent instead of relying 
          on a chroma key. This will also improve the look of fade transitions.
          <pre>
            {`.audience-root { --chroma-key-colour: rgba(0,0,0,0) !important; }\nbody { background: rgba(0,0,0,0); }`}
          </pre>
        </p>

      </Form>
    </EventWizardPageContent>
  }
}