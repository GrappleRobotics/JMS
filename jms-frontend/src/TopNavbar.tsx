import { faCompressArrowsAlt, faExpand, faHome } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import confirmBool from 'components/elements/Confirm';
import _ from 'lodash';
import { Button } from 'react-bootstrap';
import Nav from 'react-bootstrap/Nav';
import Navbar from 'react-bootstrap/Navbar';
import { Link } from 'react-router-dom';
import { WebsocketComponent } from 'support/ws-component';
import { ArenaState, LoadedMatch } from 'ws-schema';

type TopNavbarState = {
  arena_state?: ArenaState,
  match?: LoadedMatch
};

export default class TopNavbar extends WebsocketComponent<{}, TopNavbarState> {
  readonly state: TopNavbarState = {};

  componentDidMount = () => this.handles = [
    this.listen("Arena/State/Current", "arena_state"),
    this.listen("Arena/Match/Current", "match")
  ];

  triggerEstop = async () => {
    const subtitle = <p className="estop-subtitle text-muted">
      Anyone can E-Stop the match. <br />
      E-Stop if there is a safety threat or as instructed by the FTA. <br />
      <strong className="text-danger"> Robot Faults are NOT Field E-Stop conditions. </strong>
    </p>

    let result = await confirmBool(subtitle, {
      size: "xl",
      okBtn: {
        size: "lg",
        className: "estop-big",
        variant: "estop",
        children: "EMERGENCY STOP"
      },
      cancelBtn: {
        size: "lg",
        className: "btn-block",
        children: "CANCEL",
        variant: "secondary"
      }
    });

    if (result) {
      this.send({ Arena: { State: { Signal: "Estop" } } });
    }
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
    const { connected } = this.context;
    const { arena_state, match } = this.state;

    return <Navbar
      className="top-nav"
      variant="dark"
      fixed="top"
      data-connected={ connected }
      data-match-state={ match?.state }
      { ...Object.fromEntries(arena_state !== undefined ? Object.keys(arena_state).map(k => [ `data-arena-${k}`, (arena_state as any)[k] ]) : []) }
      // data-arena-state={ arena_state?.state }
    >
      <Button variant="estop" disabled={!connected || arena_state?.state === "Estop"} onClick={this.triggerEstop}>
        E-STOP
      </Button>
      <div className="me-3" />
      <Navbar.Brand>
        <strong>JMS</strong>
      </Navbar.Brand>
      <Navbar.Brand>
        { this.arenaStateText(connected, arena_state, match) }
      </Navbar.Brand>
      <Navbar.Toggle />
      <Navbar.Collapse className="justify-content-end">
        <Nav>
          <Link to="/" className="nav-link">
            <FontAwesomeIcon icon={faHome} /> &nbsp;
            Home
          </Link>
          <Nav.Link className="mx-3" onClick={ (e: any) => {
            if (fullscreen) document.exitFullscreen();
            else document.body.requestFullscreen();
            e?.preventDefault();
          }}>
            <FontAwesomeIcon icon={fullscreen ? faCompressArrowsAlt : faExpand} size="lg" />
          </Nav.Link>
        </Nav>
      </Navbar.Collapse>
    </Navbar>
  }
};