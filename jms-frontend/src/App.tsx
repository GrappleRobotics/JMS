import BottomNavbar from 'BottomNavbar';
import Debug from 'Debug';
import Home from 'Home';
import MatchControl from 'match_control/MatchControl';
import FieldMonitor from 'monitor/FieldMonitor';
import { DEBUG, ESTOPS, EVENT_WIZARD, MATCH_CONTROL, MONITOR, RANKINGS, RANKINGS_NO_SCROLL, REFEREE, REPORTS, SCORING, TIMER } from 'paths';
import Rankings from 'rankings/Rankings';
import React from 'react';
import { Col, Navbar, Row } from 'react-bootstrap';
import { Route, Routes } from 'react-router-dom';
import Reports from 'reports/Reports';
import { RefereeRouter } from 'scoring/Referee';
import { ScoringRouter } from 'scoring/Scoring';
import { WebsocketContext, WebsocketContextT } from 'support/ws-component';
import { TeamEstops } from 'TeamEstop';
import Timer from 'Timer';
import TopNavbar from 'TopNavbar';
import EventWizard from 'wizard/EventWizard';

export default class App extends React.Component {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;

  renderNoNavbar = () => {
    return this.context.connected ? <React.Fragment /> : <Navbar bg="danger" variant="dark"> <Navbar.Brand className="ml-5"> DISCONNECTED </Navbar.Brand> </Navbar>
  }

  wrapView = (children: any, props: { nonav?: boolean, fullscreen?: boolean, nopad?: boolean } = {}) => {
    let { nonav, fullscreen, nopad } = props;

    return <div className="wrapper">
      {
        nonav ? this.renderNoNavbar() : <Row className="navbar-padding">
          <Col>
            <TopNavbar />
          </Col>
        </Row>
      }
      <Row className={"app-viewport " + (fullscreen ? "fullscreen " : "") + (nopad ? "p-0 " : "")} data-connected={this.context.connected}>
        {/* <Col> */}
        { children }
        {/* </Col> */}
      </Row>
      {
        nonav ? <React.Fragment /> : <Row className="navbar-padding">
          <Col>
            <BottomNavbar />
          </Col>
        </Row>
      }
    </div>
  }

  render() {
    return <Routes>
      <Route path={EVENT_WIZARD} element={ this.wrapView(<EventWizard />) } />
      <Route path={MATCH_CONTROL} element={ this.wrapView(<MatchControl />) } />
      <Route path={MONITOR} element={ this.wrapView(<FieldMonitor />, { fullscreen: true, nopad: true }) } />
      <Route path={`${REFEREE}/*`} element={ this.wrapView(<RefereeRouter />) } />
      <Route path={`${SCORING}/*`} element={ this.wrapView(<ScoringRouter />) } />
      <Route path={RANKINGS} element={ this.wrapView(<Rankings />, { fullscreen: true, nonav: true }) } />
      <Route path={RANKINGS_NO_SCROLL} element={ this.wrapView(<Rankings scroll={false} />, { fullscreen: true }) } />
      <Route path={`${ESTOPS}/*`} element={ this.wrapView(<TeamEstops />, { fullscreen: true, nonav: true }) } />
      <Route path={DEBUG} element={ this.wrapView(<Debug />) } />
      <Route path={REPORTS} element={ this.wrapView(<Reports />) } />
      <Route path={TIMER} element={ this.wrapView(<Timer />, { nonav: true, fullscreen: true, nopad: true }) } />
      <Route path="/" element={ this.wrapView(<Home />) } />
    </Routes>
  }
};
