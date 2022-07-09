import React from 'react';
import Navbar from 'react-bootstrap/Navbar';
import Nav from 'react-bootstrap/Nav';
import { Button, Modal } from 'react-bootstrap';
import { faCompressArrowsAlt, faExpand, faHome } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { ArenaState, LoadedMatch } from 'ws-schema';
import JmsWebsocket from 'support/ws';

type TopNavbarState = {
  estop_modal: boolean,
  arena_state?: ArenaState,
  match?: LoadedMatch
};

export default class TopNavbar extends React.Component<{ ws: JmsWebsocket, connected: boolean }, TopNavbarState> {
  readonly state: TopNavbarState = {
    estop_modal: false,
  };
  handles: string[] = [];

  componentDidMount = () => {
    this.handles = [
      this.props.ws.onMessage<ArenaState>(["Arena", "State", "Current"], msg => {
        this.setState({ arena_state: msg })
      }),
      this.props.ws.onMessage<LoadedMatch | null | undefined>(["Arena", "Match", "Current"], msg => {
        this.setState({ match: msg || undefined })
      })
    ]
  }

  componentWillUnmount = () => {
    this.props.ws.removeHandles(this.handles);
  }

  estopShow = () => {
    this.setState({ estop_modal: true });
  }
  
  estopHide = () => {
    this.setState({ estop_modal: false });
  }

  triggerEstop = () => {
    this.props.ws.send({ Arena: { State: { Signal: "Estop" } } });
    this.estopHide();
  }

  estopModal = () => {
    return (
      <Modal 
        show={this.state.estop_modal}
        onHide={this.estopHide}
        backdrop="static"
        animation={false}
        centered
        size="lg"
      >
        <Modal.Body>
          <p className="estop-subtitle text-muted">
            Anyone can E-Stop the match. <br />
            E-Stop if there is a safety threat, field fault / broken, or instructed by the FTA. <br />
            <strong className="text-danger"> Robot Faults are NOT Field E-Stop conditions. </strong>
          </p>
          <div className="my-5" />
          <Button
            size="lg"
            className="estop-big"
            block
            variant="hazard-red-dark"
            onClick={this.triggerEstop}
          >
            EMERGENCY STOP
          </Button>
          <div className="my-5" />
          <Button block variant="secondary" onClick={this.estopHide}> CANCEL </Button>
        </Modal.Body>
      </Modal>);
  }

  arenaStateText = (connected: boolean, state?: ArenaState, match?: LoadedMatch) => {
    if (!connected) return "DISCONNECTED";
    switch (state?.state) {
      case "Idle":
        return state.ready ? "Idle" : "Idle (working...)";
      case "Prestart":
        return state.ready ? "Prestart Ready" : "Prestarting...";
      case "MatchArmed":
        return "ARMED";
      case "MatchPlay":
        switch (match?.state) {
          case "Auto":
            return `Auto (T- ${match.remaining_time.secs}s)`;
          case "Teleop":
            return `Teleop (T- ${match.remaining_time.secs}s)`;
          default:
            return match?.state;
        }
      case "MatchComplete":
        return state.ready ? "Match Complete" : "Match Completed (score wait)";
      case "MatchCommit":
        return "Scores Committed";
      case "Estop":
        return "// EMERGENCY STOP //";
      default:
        return state?.state;
    }
  }

  render() {
    let fullscreen = document.fullscreenElement != null;
    const { connected } = this.props;
    const { arena_state, match } = this.state;

    return <Navbar variant="dark" fixed="top">
      <Button variant="hazard-red-dark" disabled={!connected || arena_state?.state === "Estop"} onClick={this.estopShow}>
        E-STOP
      </Button>
      <div className="mr-3" />
      <Navbar.Brand>
        <strong>JMS</strong>
      </Navbar.Brand>
      <Navbar.Brand data-connected={ connected } data-arena-state={ arena_state?.state } data-match-state={ match?.state }>
        { this.arenaStateText(connected, arena_state, match) }
      </Navbar.Brand>
      <Navbar.Toggle />
      <Navbar.Collapse className="justify-content-end">
        <Nav>
          <Nav.Link href="/">
            <FontAwesomeIcon icon={faHome} /> &nbsp;
            Home
          </Nav.Link>
          <Nav.Link className="mx-3" onClick={ (e: any) => {
            if (fullscreen) document.exitFullscreen();
            else document.body.requestFullscreen();
            e?.preventDefault();
          }}>
            <FontAwesomeIcon icon={fullscreen ? faCompressArrowsAlt : faExpand} size="lg" />
          </Nav.Link>
        </Nav>
      </Navbar.Collapse>

      <this.estopModal />
    </Navbar>
  }
};