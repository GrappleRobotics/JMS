import { faExclamationTriangle, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Col, Container, Nav, Row, Tab } from "react-bootstrap";
import ConfigureAlliances from "./ConfigureAlliances";
import ConfigureEvent from "./ConfigureEvent";
import ConfigureSchedule from "./ConfigureSchedule";
import ConfigureTeams from "./ConfigureTeams";
import QualGenerator from "./QualGenerator";

const EK_WELCOME = 'welcome';

function Welcome() {
  return <div>
    <h4>Welcome to the Event Wizard</h4>
    <br/>
    <p> Welcome to the Event Wizard. The Event Wizard goes through event configuration step-by-step to ensure your event runs appropriately. </p>
    <p> Select from the tabs on the left to start configuring your event, starting with the event details. More options will become available as the event
      progresses. </p>
    <p className="text-muted"> <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; <i> You can also use JMS without loading event data to run test matches only. </i> </p>
  </div>
}

export default class EventWizard extends React.Component {
  constructor(props) {
    super(props);

    this.state = {};

    props.ws.subscribe("event", "*");
    props.ws.subscribe("matches", "*");
  }

  render() {
    let { event, matches, ws } = this.props;
    let { details, teams, schedule, alliances } = event || {};

    let data = {
      details, teams, schedule, matches, alliances
    };

    let navItemFor = (data, cls) => {
      let disabled = cls.isDisabled?.(data) || false;
      let attention = !disabled && (cls.needsAttention?.(data) || false);

      return <Nav.Item>
        <Nav.Link className={ attention ? "wizard-attention" : "" } eventKey={cls.eventKey()} disabled={disabled}> 
          { attention ? <FontAwesomeIcon icon={faExclamationTriangle} /> : "" } &nbsp;
          { cls.tabName(data) }
        </Nav.Link>
      </Nav.Item>
    };

    let paneFor = (el) => <Tab.Pane eventKey={el.type.eventKey()}>
      {el}
    </Tab.Pane>
    
    return <Container fluid className="px-5">
      <h3>Event Wizard { event?.event_name ? ("- " + event.event_name) : "" }</h3>
      <hr />
      <Tab.Container defaultActiveKey={EK_WELCOME}>
        <Row>
          <Col md={3} className="vr-right wizard-tabs">
            <Nav variant="pills" className="flex-column">
              <Nav.Item> <Nav.Link eventKey={EK_WELCOME}> &nbsp;Welcome! </Nav.Link> </Nav.Item>

              <br /> <h6 className="text-muted">Pre-Event Config</h6>
              { navItemFor(data, ConfigureEvent) }
              { navItemFor(data, ConfigureTeams) }
              { navItemFor(data, ConfigureSchedule) }

              <br /> <h6 className="text-muted">Qualifications</h6>
              { navItemFor(data, QualGenerator) }

              <br /> <h6 className="text-muted">Playoffs</h6>
              { navItemFor(data, ConfigureAlliances) }
              <br /> <h6 className="text-muted">Awards</h6>
            </Nav>
          </Col>
          <Col md>
            <Tab.Content>
              <br />
              <Tab.Pane eventKey={EK_WELCOME}> <Welcome /> </Tab.Pane>
              { paneFor(<ConfigureEvent {...data} ws={ws} />) }
              { paneFor(<ConfigureTeams {...data} ws={ws} />) }
              { paneFor(<ConfigureSchedule {...data} ws={ws} />) }
              { paneFor(<QualGenerator {...data} ws={ws} />) }
              { paneFor(<ConfigureAlliances {...data} ws={ws} />) }
              <br />
            </Tab.Content>
          </Col>
        </Row>
      </Tab.Container>
    </Container>
  }
}