import { faCompressArrowsAlt, faCrown, faExpand, faHome, faRightFromBracket } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import BufferedFormControl from 'components/elements/BufferedFormControl';
import confirmBool, { confirmModal } from 'components/elements/Confirm';
import _ from 'lodash';
import React from 'react';
import { Button, FormControl, Modal } from 'react-bootstrap';
import Nav from 'react-bootstrap/Nav';
import Navbar from 'react-bootstrap/Navbar';
import { Link } from 'react-router-dom';
import { role2id } from 'support/ws-additional';
import { WebsocketComponent } from 'support/ws-component';
import { ArenaState, LoadedMatch, SerializedMatch, TaggedResource } from 'ws-schema';

type TopNavbarState = {
  arena_state?: ArenaState,
  match?: LoadedMatch,
  resource?: TaggedResource,
  readyModal: boolean
};

export default class TopNavbar extends WebsocketComponent<{}, TopNavbarState> {
  readonly state: TopNavbarState = { readyModal: false };

  componentDidMount = () => this.handles = [
    this.listen("Arena/State/Current", "arena_state"),
    this.listen("Arena/Match/Current", "match"),
    this.listen("Resource/Current", "resource")
  ];

  componentDidUpdate = (prevProps: {}, prevState: TopNavbarState) => {
    if (this.state.resource?.ready_requested && !prevState.resource?.ready_requested && !this.state.resource?.ready && !this.state.readyModal) {
      this.setState({ readyModal: true });
    }
  }

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

  tryFTA = () => {
    confirmModal("", {
      data: "",
      title: "FTA Login",
      okText: "Login",
      renderInner: (pass: string, onUpdate, ok) => <React.Fragment>
        <h6> Enter FTA PIN: </h6>
        <BufferedFormControl
          instant
          autofocus
          type="password"
          value={pass}
          onUpdate={(v) => onUpdate(String(v))}
          onEnter={ok}
        />
      </React.Fragment>
    }).then(pass => this.send({ Resource: { SetFTA: pass } }))
  }

  render() {
    let fullscreen = document.fullscreenElement != null;
    const { connected } = this.context;
    const { arena_state, match, resource, readyModal } = this.state;

    return <Navbar
      className="top-nav"
      variant="dark"
      fixed="top"
      data-fta={ resource?.fta }
      data-role={ resource ? role2id(resource.role) : undefined }
      data-ready-required={ resource?.ready_requested }
      data-ready={ resource?.ready }
      data-connected={ connected }
      data-match-state={ match?.state }
      { ...Object.fromEntries(arena_state !== undefined ? Object.keys(arena_state).map(k => [ `data-arena-${k}`, (arena_state as any)[k] ]) : []) }
      // data-arena-state={ arena_state?.state }
    >
      <Button className="me-3" variant="estop" disabled={!connected || arena_state?.state === "Estop"} onClick={this.triggerEstop}>
        E-STOP
      </Button>
      {
        resource?.ready_requested ? 
          <Button
            className="me-3"
            variant={resource?.ready ? "bad" : "good"}
            onClick={() => this.send({ Resource: { SetReady: !resource?.ready } })}
          > { resource?.ready ? "Cancel Ready" : "SET READY" } </Button> : undefined
      }
      <Navbar.Brand>
        <strong>JMS</strong>
      </Navbar.Brand>
      <Navbar.Brand>
        { this.arenaStateText(connected, arena_state, match) }
      </Navbar.Brand>
      <Navbar.Toggle />
      <Navbar.Collapse className="justify-content-end">
        <Nav>
          <Link to="/" className="nav-link mx-3">
            <FontAwesomeIcon icon={faHome} /> &nbsp;
            Home
          </Link>
          {
            resource?.fta ? <React.Fragment>
              <Navbar.Brand className="brand-fta"> <strong>FTA</strong> </Navbar.Brand>
              <Nav.Link onClick={() => this.send({ Resource: { SetFTA: null } })}>
                <FontAwesomeIcon icon={faRightFromBracket} /> Logout
              </Nav.Link>
            </React.Fragment> 
            : <Nav.Link onClick={this.tryFTA}>
                <FontAwesomeIcon icon={faCrown} /> FTA Login
              </Nav.Link>
          }
          <Nav.Link className="mx-3" onClick={ (e: any) => {
            if (fullscreen) document.exitFullscreen();
            else document.body.requestFullscreen();
            e?.preventDefault();
          }}>
            <FontAwesomeIcon icon={fullscreen ? faCompressArrowsAlt : faExpand} size="lg" />
          </Nav.Link>
        </Nav>
      </Navbar.Collapse>

      <Modal show={readyModal} centered size="lg">
        <Modal.Header> <Modal.Title> Are you ready? </Modal.Title> </Modal.Header>
        <Modal.Body>
          Before { match?.match_meta?.name || "the match" } can begin, you're required to notify the scorekeeper that you are <strong className="text-good"> READY </strong>
        </Modal.Body>
        <Modal.Footer>
          <Button className="ready-big" variant="good" onClick={() => { this.send({ Resource: { SetReady: true } }); this.setState({ readyModal: false }) }}>
            I'M READY
          </Button>
          <Button className="ready-later" variant="secondary" onClick={() => this.setState({ readyModal: false })}>
            LATER
          </Button>
        </Modal.Footer>
      </Modal>
    </Navbar>
  }
};