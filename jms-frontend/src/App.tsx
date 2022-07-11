import React from 'react';
import { Route, Routes } from 'react-router-dom';
import JmsWebsocket from 'support/ws';
import MatchControl from 'match_control/MatchControl';
import EventWizard from 'wizard/EventWizard';
import { AUDIENCE, AUDIENCE_CONTROL, DEBUG, ESTOPS, EVENT_WIZARD, MATCH_CONTROL, MONITOR, RANKINGS, RANKINGS_NO_SCROLL, REFEREE, REPORTS, SCORING, TIMER } from 'paths';
import TopNavbar from 'TopNavbar';
import { Col, Navbar, Row } from 'react-bootstrap';
import BottomNavbar from 'BottomNavbar';
import { nullIfEmpty } from 'support/strings';
import Home from 'Home';
import { ScoringRouter } from 'scoring/Scoring';
import Rankings from 'rankings/Rankings';
import FieldMonitor from 'monitor/FieldMonitor';
import Reports from 'reports/Reports';
import Audience from 'audience/Audience';
import AudienceDisplayControl from 'audience/AudienceDisplayControl';
import { RefereeRouter } from 'scoring/Referee';
import Debug from 'Debug';
import Timer from 'Timer';
import { TeamEstops } from 'TeamEstop';

type AppState = {
  connected: boolean
};

export default class App extends React.Component< { ws: JmsWebsocket }, AppState > {
  readonly state: AppState = {
    connected: false
  };

  componentDidMount = () => {
    this.props.ws.onConnectChange(connected => this.setState({ connected }))
  }

  renderNoNavbar = () => {
    return this.state.connected ? <React.Fragment /> : <Navbar bg="danger" variant="dark"> <Navbar.Brand className="ml-5"> DISCONNECTED </Navbar.Brand> </Navbar>
  }

  wrapView = (children: any, props: { nonav?: boolean, fullscreen?: boolean, nopad?: boolean } = {}) => {
    let { nonav, fullscreen, nopad } = props;

    return <div className="wrapper">
      {
        nonav ? this.renderNoNavbar() : <Row className="navbar-padding">
          <Col>
            <TopNavbar
              ws={this.props.ws}
              connected={this.state.connected}
            />
          </Col>
        </Row>
      }
      <Row className={"app-viewport " + (fullscreen ? "fullscreen " : "") + (nopad ? "p-0 " : "")} data-connected={this.state.connected}>
        {/* <Col> */}
        { children }
        {/* </Col> */}
      </Row>
      {
        nonav ? <React.Fragment /> : <Row className="navbar-padding">
          <Col>
            <BottomNavbar
              ws={this.props.ws}
            />
          </Col>
        </Row>
      }
    </div>
  }

  render() {
    // let { arena, event, matches } = this.state;

      {/* <Route path={EVENT_WIZARD}>
        <this.wrapView navbar>
          <EventWizard
            ws={this.ws}
            event={event}
            matches={matches}
          />
        </this.wrapView>
      </Route>
      <Route path={SCORING}>
        <this.wrapView navbar fullscreen>
          <ScoringRouter
            ws={this.ws}
            arena={arena}
          />
        </this.wrapView>
      </Route>
      <Route path={REPORTS}>
        <this.wrapView navbar>
          <Reports />
        </this.wrapView>
      </Route>
      <Route path={MATCH_CONTROL}>
        <this.wrapView navbar>
          <MatchControl
            ws={this.ws}
            arena={arena}
            matches={matches}
          />
        </this.wrapView>
      </Route>
      <Route path={RANKINGS_NO_SCROLL}>
        <this.wrapView fullscreen>
          <Rankings
            ws={this.ws}
            rankings={event?.rankings}
            details={event?.details}
            next_match={matches?.next}
            scroll={false}
          />
        </this.wrapView>
      </Route>
      <Route path={RANKINGS}>
        <this.wrapView fullscreen>
          <Rankings
            ws={this.ws}
            rankings={event?.rankings}
            details={event?.details}
            next_match={matches?.next}
            scroll={true}
          />
        </this.wrapView>
      </Route>
      <Route path={AUDIENCE_CONTROL}>
        <this.wrapView>
          <AudienceDisplayControl
            ws={this.ws}
            event={event}
          />
        </this.wrapView>
      </Route>
      <Route path={AUDIENCE}>
        <this.wrapView fullscreen>
          <Audience
            ws={this.ws}
            arena={arena}
            event={event}
            matches={matches}
          />
        </this.wrapView>
      </Route>
      <Route path={REFEREE}>
        <this.wrapView navbar>
          <RefereeRouter
            ws={this.ws}
            arena={arena}
          />
        </this.wrapView>
      </Route>
      <Route path={MONITOR}>
        <this.wrapView navbar fullscreen nopad>
          <FieldMonitor
            ws={this.ws}
            arena={arena}
          />
        </this.wrapView>
      </Route>
      <Route path={TIMER}>
        <this.wrapView fullscreen nopad>
          <Timer
            ws={this.ws}
            arena={arena}
          />
        </this.wrapView>
      </Route>
      <Route path={ESTOPS}>
        <this.wrapView fullscreen>
          <TeamEstops
            ws={this.ws}
            arena={arena}
          />
        </this.wrapView>
      </Route>
      <Route path={DEBUG}>
        <this.wrapView navbar>
          <Debug
            ws={this.ws}
          />
        </this.wrapView>
      </Route> */}
    return <Routes>
      <Route path={EVENT_WIZARD} element={ this.wrapView(<EventWizard ws={this.props.ws} />) } />
      <Route path={MATCH_CONTROL} element={ this.wrapView(<MatchControl ws={this.props.ws} />) } />
      <Route path={ESTOPS} element={ this.wrapView(<TeamEstops ws={this.props.ws} />, { fullscreen: true }) } />
      <Route path={DEBUG} element={ this.wrapView(<Debug ws={this.props.ws} />) } />
      <Route path={REPORTS} element={ this.wrapView(<Reports />) } />
      <Route path={TIMER} element={ this.wrapView(<Timer ws={this.props.ws} />, { nonav: true, fullscreen: true, nopad: true }) } />
      <Route path="/" element={ this.wrapView(<Home />) } />
    </Routes>
  }
};
