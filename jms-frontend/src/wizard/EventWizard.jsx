import { faExclamationTriangle, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Col, Container, Nav, Row, Tab } from "react-bootstrap";
import ConfigureEvent from "./S1_ConfigureEvent";
import ConfigureTeams from "./S2_ConfigureTeams";

const EK_WELCOME = 'welcome';

function Welcome() {
  return <div>
    <h4>Welcome to the Event Wizard</h4>
    <br/>
    <p> No data for this event has been loaded yet. Using the tabs on the left, load the event
      details in the order listed to get your event up and running. </p>
    <p> After you start to configure your event, JMS will highlight parts of the event wizard that require action. </p>
    <p className="text-muted"> <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; <i> You can also use JMS without loading event data to run test matches only. </i> </p>
  </div>
}

export default class EventWizard extends React.Component {
  constructor(props) {
    super(props);

    this.state = {};
  }

  render() {
    let {event, teams, ws} = this.props;

    let navItemFor = (data, cls) => {
      let disabled = cls.isDisabled?.(data) || false;
      let attention = cls.needsAttention?.(data) || false;

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
      <h2>Event Wizard</h2>
      <hr />
      <Tab.Container defaultActiveKey={EK_WELCOME}>
        <Row>
          <Col md={3} className="vr-right wizard-tabs">
            <Nav variant="pills" className="flex-column">
              <Nav.Item> <Nav.Link eventKey={EK_WELCOME}> &nbsp;Welcome! </Nav.Link> </Nav.Item>
              { navItemFor(event, ConfigureEvent) }
              { navItemFor(teams, ConfigureTeams) }
            </Nav>
          </Col>
          <Col md>
            <Tab.Content>
              <br />
              <Tab.Pane eventKey={EK_WELCOME}> <Welcome /> </Tab.Pane>
              { paneFor(<ConfigureEvent event={event} ws={ws} />) }
              { paneFor(<ConfigureTeams teams={teams} ws={ws} />) }
              <br />
            </Tab.Content>
          </Col>
        </Row>
      </Tab.Container>
    </Container>
  }
}