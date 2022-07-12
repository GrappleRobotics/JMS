import { faExclamationTriangle, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Button, Col, Container, Nav, Row, Tab, TabProps, Tabs } from "react-bootstrap";
import { EventDetails, Team, ScheduleBlock, PlayoffAlliance, TeamRanking, Award } from "ws-schema";
import ConfigureAlliances from "./ConfigureAlliances";
import ConfigureAwards from "./ConfigureAwards";
import ConfigureEvent from "./ConfigureEvent";
import ConfigureSchedule from "./ConfigureSchedule";
import ConfigureTeams from "./ConfigureTeams";
import PlayoffGenerator from "./PlayoffGenerator";
import QualGenerator from "./QualGenerator";

interface EventWizardPageMeta {
  tabLabel: React.ReactNode,
  attention: boolean,
  disabled: boolean
};

type EventWizardPageContextType = {
  renderInner: boolean,
  key: string,
  onMetaUpdate?: (key: string, meta: EventWizardPageMeta) => void
}

const EventWizardPageContext = React.createContext({} as EventWizardPageContextType);

type EventWizardPageContentProps = {
  children: React.ReactNode
} & EventWizardPageMeta;

export class EventWizardPageContent extends React.Component<EventWizardPageContentProps> {
  static defaultProps = { attention: false, disabled: false }
  static contextType = EventWizardPageContext;
  context!: React.ContextType<typeof EventWizardPageContext>;

  updateContext = () => {
    if (this.context.onMetaUpdate) {
      this.context.onMetaUpdate(this.context.key, {
        tabLabel: this.props.tabLabel, 
        attention: this.props.attention, 
        disabled: this.props.disabled
      });
    }
  }

  componentDidMount = () => {
    this.updateContext();
  }

  componentDidUpdate = (prev: EventWizardPageMeta) => {
    if (prev.tabLabel !== this.props.tabLabel || prev.attention !== this.props.attention || prev.disabled !== this.props.disabled) {
      this.updateContext();
    }
  }

  render() {
    return this.context.renderInner ? this.props.children : <React.Fragment />;
  }
}

export class Welcome extends React.PureComponent {
  render() {
    return <EventWizardPageContent tabLabel="Welcome">
      <h4>Welcome to the Event Wizard</h4>
      <br/>
      <p> Welcome to the Event Wizard. The Event Wizard goes through event configuration step-by-step to ensure your event runs appropriately. </p>
      <p> Select from the tabs on the left to start configuring your event, starting with the event details. More options will become available as the event
        progresses. </p>
      <p className="text-muted"> <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; <i> You can also use JMS without loading event data to run test matches only. </i> </p>
    </EventWizardPageContent>
  }
}

type EventWizardMetaState = {
  welcome?: EventWizardPageMeta,
  configEvent?: EventWizardPageMeta,
  configTeams?: EventWizardPageMeta
  configSchedule?: EventWizardPageMeta
  qualGen?: EventWizardPageMeta
}

type EventWizardState = {
  active: string,
  metas: EventWizardMetaState
}

export default class EventWizard extends React.Component<{}, EventWizardState> {
  readonly state: EventWizardState = {
    active: "welcome",
    metas: {}
  };

  updateMeta = (key: string, meta: EventWizardPageMeta) => {
    this.setState((state) => ({
      metas: { ...state.metas, [key]: meta }
    }))
  }

  navFor = (key: keyof EventWizardMetaState) => {
    let { tabLabel, attention, disabled } = this.state.metas[key] || { tabLabel: "Unknown", attention: false, disabled: false };
    return <Nav.Item>
      <Nav.Link className="wizard-tab-link" eventKey={key} disabled={disabled} data-attention={!disabled && attention}>
        { (attention && !disabled) ? <FontAwesomeIcon icon={faExclamationTriangle} /> : "" } &nbsp;
        { tabLabel }
      </Nav.Link>
    </Nav.Item>
  }

  paneFor = (key: keyof EventWizardMetaState, component: React.ReactElement) => {
    return <Tab.Pane eventKey={key}>
        <EventWizardPageContext.Provider value={ { key: key, renderInner: this.state.active === key, onMetaUpdate: this.updateMeta } }>
          { component }
        </EventWizardPageContext.Provider>
    </Tab.Pane>;
  }

  render() {
    return <Container fluid className="px-5">
      <h3>Event Wizard</h3>
      <hr />
      <Tab.Container activeKey={this.state.active} unmountOnExit={false} onSelect={(active) => this.setState({ active: active || "test1" })}>
        <Row>
          <Col md={3} className="vr-right wizard-tabs">
            <Nav variant="pills" className="flex-column">
              { this.navFor("welcome") }  
              <br /> <h6 className="text-muted">Pre-Event Config</h6>
              { this.navFor("configEvent") }
              { this.navFor("configTeams") }
              { this.navFor("configSchedule") }
              <br /> <h6 className="text-muted">Qualifications</h6>
              { this.navFor("qualGen") }
              <br /> <h6 className="text-muted">Playoffs</h6>
              <br /> <h6 className="text-muted">Awards</h6>
            </Nav>
          </Col>
          <Col md>
            <Tab.Content>
              <br />
              { this.paneFor("welcome", <Welcome />) }
              { this.paneFor("configEvent", <ConfigureEvent />) }
              { this.paneFor("configTeams", <ConfigureTeams />) }
              { this.paneFor("configSchedule", <ConfigureSchedule />) }
              { this.paneFor("qualGen", <QualGenerator />) }
            </Tab.Content>
          </Col>
        </Row>
      </Tab.Container>
    </Container>
  }
}
