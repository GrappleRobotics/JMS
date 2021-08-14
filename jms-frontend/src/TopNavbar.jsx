import React from 'react';
import Navbar from 'react-bootstrap/Navbar';
import Nav from 'react-bootstrap/Nav';
import { Button, Modal } from 'react-bootstrap';
import { faCompressArrowsAlt, faExpand, faHome, faMagic } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

export default class TopNavbar extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      estop_modal: false
    }

    props.ws.subscribe("arena", "state");
  }

  estopShow = () => {
    this.setState({ estop_modal: true });
  }
  
  estopHide = () => {
    this.setState({ estop_modal: false });
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
            onClick={() => { this.props.onEstop(); this.estopHide() }}
          >
            EMERGENCY STOP
          </Button>
          <div className="my-5" />
          <Button block variant="secondary" onClick={this.estopHide}> CANCEL </Button>
        </Modal.Body>
      </Modal>);
  }

  decodeArenaState = () => {
    let connected = this.props.connected;
    let state = this.props.state;
    let match = this.props.match;

    if (!connected || state == null)
      return ["DISCONNECTED", "danger"];
    
    switch (state.state) {
      case "Idle":
        return state.ready ? ["Idle", "dark"] : ["Idle (working)...", "warning"];
      case "Prestart":
        return state.ready ? ["Prestarted", "success"] : ["Prestarting...", "warning"];
      case "MatchArmed":
        return [<strong>ARMED</strong>, "hazard-dark-active"];
      case "MatchPlay":
        switch (match.state) {
          case "Warmup":
          case "Pause":
          case "Cooldown":
            return [match.state, "dark"];
          case "Auto":
            return ["Autonomous (T- " + match.remaining_time.secs + "s)", "info"]
          case "Teleop":
            return ["Teleop (T- " + match.remaining_time.secs + "s)", "info"];
          case "Fault":
            return [<strong>FAULT</strong>, "danger"];
          default:
            return [match.state, "primary"];
        }
      case "MatchComplete":
        return ["Match Complete", "success"];
      case "MatchCommit":
        return ["Scores Commited", "success"];
      case "Estop":
        return [<strong>// EMERGENCY STOP //</strong>, "hazard-red-dark-active"]
      default:
        return [state.state, "dark"];
    }
  };

  render() {
    const [arenaState, navbarColour] = this.decodeArenaState();
    let fullscreen = document.fullscreenElement != null;

    return <Navbar bg={navbarColour} variant="dark" fixed="top">
      <Button variant="hazard-red-dark" disabled={!this.props.connected || this.props.state?.state == "Estop"} onClick={this.estopShow}>
        E-STOP
      </Button>
      <div className="mr-3" />
      <Navbar.Brand>
        <strong>JMS</strong>
      </Navbar.Brand>
      <Navbar.Brand>
        { arenaState }
      </Navbar.Brand>
      <Navbar.Toggle />
      <Navbar.Collapse className="justify-content-end">
        <Nav>
          <Nav.Link href="/">
            <FontAwesomeIcon icon={faHome} /> &nbsp;
            Home
          </Nav.Link>
          <Nav.Link className="mx-3" onClick={ e => {
            if (fullscreen) document.exitFullscreen();
            else document.body.requestFullscreen();
            e.preventDefault();
          }}>
            <FontAwesomeIcon icon={fullscreen ? faCompressArrowsAlt : faExpand} size="lg" />
          </Nav.Link>
        </Nav>
      </Navbar.Collapse>

      <this.estopModal />
    </Navbar>
  }
};