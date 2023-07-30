import React from "react";
import { WebsocketComponent } from "support/ws-component";

type AppState = {
  errors: String[],
  fta: boolean
};

export default class App extends WebsocketComponent<{}, AppState> {
  readonly state: AppState = { errors: [], fta: false };
  private topnavRef = React.createRef<HTMLDivElement>();
  private bottomnavRef = React.createRef<HTMLDivElement>();

  componentDidMount = () => this.handles = [
    this.listenFn("Resource/Current", (r: TaggedResource) => this.setState({ fta: r.fta || false })),
    this.listenFn("Error", (err: string) => this.setState(s => update(s, { errors: { $push: [err] } })))
  ];

  renderNoNavbar = () => {
    return this.context.connected ? <React.Fragment /> : <Navbar bg="danger" variant="dark"> <Navbar.Brand className="ml-5"> DISCONNECTED </Navbar.Brand> </Navbar>
  }

  wrapView = (children: any, props: { nonav?: boolean, fullscreen?: boolean, nopad?: boolean } = {}) => {
    let { nonav, fullscreen, nopad } = props;

    return <div className="wrapper">
      {
        nonav ? this.renderNoNavbar() : <Row className="navbar-padding-top" style={ { "--nav-height": `${this.topnavRef.current?.clientHeight}px` } as React.CSSProperties }>
          <Col>
            <TopNavbar innerRef={this.topnavRef} />
          </Col>
        </Row>
      }

      {
        this.state.errors.map((e, i) => <Alert dismissible variant="danger" onClose={() => this.setState(s => update(s, { errors: { $splice: [[i, 1]] } }))}>
          Error: { e }
        </Alert>)
      }

      <Row className={"app-viewport " + (fullscreen ? "fullscreen " : "") + (nopad ? "p-0 " : "")} data-connected={this.context.connected}>
        {/* <Col> */}
        { children }
        {/* </Col> */}
      </Row>
      {
        nonav ? <React.Fragment /> : <Row className="navbar-padding-bottom" style={ { "--nav-height": `${this.bottomnavRef.current?.clientHeight}px` } as React.CSSProperties }>
          <Col>
            <BottomNavbar innerRef={this.bottomnavRef} />
          </Col>
        </Row>
      }
    </div>
  }

  render() {
    const fta = this.state.fta;
    return <Routes>
      <Route path={EVENT_WIZARD} element={ this.wrapView(<EventWizard fta={fta} />) } />
      <Route path={MATCH_CONTROL} element={ withRole("ScorekeeperPanel", this.wrapView(<MatchControl fta={fta} />)) } />
      <Route path={MONITOR} element={ withRole("MonitorPanel", this.wrapView(<FieldMonitor fta={fta} />, { fullscreen: true, nopad: true })) } />
      <Route path={FTA} element={ this.wrapView(<FTAView fta={fta} />, { fullscreen: true, nopad: true }) } />
      <Route path={AUDIENCE_CONTROL} element={ this.wrapView(<AudienceDisplayControl />) } />
      <Route path={`${REFEREE}/*`} element={ this.wrapView(<RefereeRouter />) } />
      <Route path={`${SCORING}/*`} element={ this.wrapView(<ScoringRouter />) } />
      <Route path={AUDIENCE} element={ withRole("AudienceDisplay", this.wrapView(<Audience />, { fullscreen: true, nonav: true })) } />
      <Route path={RANKINGS} element={ this.wrapView(<Rankings />, { fullscreen: true, nonav: true }) } />
      <Route path={RANKINGS_NO_SCROLL} element={ this.wrapView(<Rankings scroll={false} />, { fullscreen: true }) } />
      <Route path={`${ESTOPS}/*`} element={ this.wrapView(<TeamEstops />, { fullscreen: true, nonav: true }) } />
      <Route path={DEBUG} element={ this.wrapView(<Debug fta={fta} />) } />
      <Route path={REPORTS} element={ this.wrapView(<Reports fta={fta} />) } />
      <Route path={`${CSA}/*`} element={ this.wrapView(<CSARouter fta={fta} />) } />
      <Route path={LOGS} element={ this.wrapView(<MatchLogs />) } />
      <Route path={TIMER} element={ withRole("TimerPanel", this.wrapView(<Timer />, { nonav: true, fullscreen: true, nopad: true })) } />
      <Route path="/" element={ this.wrapView(<Home fta={fta} />) } />
    </Routes>
  }
};