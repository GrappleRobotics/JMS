import React from 'react';
import Navbar from 'react-bootstrap/Navbar';
import Nav from 'react-bootstrap/Nav';
import { Button } from 'react-bootstrap';
import { EVENT_WIZARD, MATCH_CONTROL } from 'paths';

export default class NavBar extends React.Component {
  decodeArenaState = () => {
    let connected = this.props.connected;
    let state = this.props.state;
    let match = this.props.match;

    if (!connected || state == null)
      return ["DISCONNECTED", "danger"];
    
    switch (state.state) {
      case "Idle":
        return ["Idle", match ? "dark" : "secondary"];
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

  renderMatch = () => {
    let match = this.props.match;
    if (match) {
      if (match.meta.type == "Test") {
        return <i>Test Match</i>;
      } else {
        return match.meta.name;
      }
    } else {
      return <i>No Match Loaded</i>;
    }
  }

  render() {
    const [arenaState, navbarColour] = this.decodeArenaState();
    return <Navbar bg={navbarColour} variant="dark">
      <Button variant="hazard-red-dark" disabled={!this.props.connected || this.props.state?.state == "Estop"} onClick={this.props.onEstop}>
        E-STOP
      </Button>
      <div className="mr-3" />
      <Navbar.Brand>
        <strong>JMS</strong>
      </Navbar.Brand>
      <Navbar.Brand>
        { arenaState }
      </Navbar.Brand>
      <Navbar.Brand>
        { this.renderMatch() }
      </Navbar.Brand>
      <Navbar.Toggle />
      <Navbar.Collapse className="justify-content-end">
        <Nav>
          <Nav.Link href={EVENT_WIZARD}>Wizard</Nav.Link>
          <Nav.Link href={MATCH_CONTROL}>Match Control</Nav.Link>
        </Nav>
      </Navbar.Collapse>
    </Navbar>
  }
};