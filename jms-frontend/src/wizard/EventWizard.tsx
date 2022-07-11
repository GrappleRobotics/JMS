import { faExclamationTriangle, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Button, Col, Container, Nav, Row, Tab, TabProps, Tabs } from "react-bootstrap";
import JmsWebsocket from "support/ws";
import { EventDetails, Team, ScheduleBlock, PlayoffAlliance, TeamRanking, Award } from "ws-schema";
import ConfigureAlliances from "./ConfigureAlliances";
import ConfigureAwards from "./ConfigureAwards";
import ConfigureEvent from "./ConfigureEvent";
import ConfigureSchedule from "./ConfigureSchedule";
import ConfigureTeams from "./ConfigureTeams";
import PlayoffGenerator from "./PlayoffGenerator";
import QualGenerator from "./QualGenerator";

const EK_WELCOME = 'welcome';

// type EventWizardTabProps = {
//   name: string
//   attention?: boolean,
//   disabled?: boolean,
//   children: React.ReactNode
// }

// export class EventWizardTab extends React.PureComponent<EventWizardTabProps> {
//   render() {

//   }
// }

// interface EventWizardPageProps {
//   key: string,
//   attention: boolean,
//   disabled: boolean,
//   tab: React.ReactNode,
//   children: React.ReactNode
// }

// export class EventWizardPage extends React.PureComponent<EventWizardPageProps> {
//   class Content extends React.PureComponent<> {

//   }

//   render() {
//     return <div className="event-wizard-page">
//       { this.props.children }
//     </div>
//   }
// }

// interface EventWizardPageType {
//   tabName: string,
//   needsAttention: boolean
//   isDisabled: boolean
// }


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
  configEvent?: EventWizardPageMeta
}

type EventWizardState = {
  active: string,
  metas: EventWizardMetaState
}

export default class EventWizard extends React.Component<{ ws: JmsWebsocket }, EventWizardState> {
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
      <Nav.Link className="wizard-tab-link" eventKey={key} disabled={disabled} data-attention={attention}>
        { attention ? <FontAwesomeIcon icon={faExclamationTriangle} /> : "" } &nbsp;
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
    let { ws } = this.props;

    return <Container fluid className="px-5">
      <h3>Event Wizard</h3>
      <hr />
      <Tab.Container activeKey={this.state.active} unmountOnExit={false} onSelect={(active) => this.setState({ active: active || "test1" })}>
        <Row>
          <Col md={3} className="vr-right wizard-tabs">
            <Nav variant="pills" className="flex-column">
              { this.navFor("welcome") }  
              { this.navFor("configEvent") }  
            </Nav>
          </Col>
          <Col md>
            <Tab.Content>
              <br />
              { this.paneFor("welcome", <Welcome />) }
              { this.paneFor("configEvent", <ConfigureEvent ws={ws} />) }
            </Tab.Content>
          </Col>
        </Row>
      </Tab.Container>
    </Container>
  }
}
