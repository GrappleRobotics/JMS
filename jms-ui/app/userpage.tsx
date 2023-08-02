"use client";
import { useWebsocket } from "./support/ws-component";
import React, { useEffect, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faRightFromBracket } from "@fortawesome/free-solid-svg-icons";
import { Alert, Button, Col, Container, Nav, Navbar, Row } from "react-bootstrap";
import "./userpage.scss";
import Link from "next/link";
import { useErrors } from "./support/errors";
import { ArenaState, JmsComponent, SerialisedLoadedMatch } from "./ws-schema";
import { PermissionGate } from "./support/permissions";
import confirmBool from "./components/Confirm";
import JmsWebsocket from "./support/ws";
import moment from "moment";
import SimpleTooltip from "./components/SimpleTooltip";

interface UserPageProps {
  container?: boolean,
  space?: boolean,
  children: React.ReactNode
};

export default function UserPage(props: UserPageProps) {
  const { error, removeError } = useErrors();

  const alert = error !== null && <Alert className="m-3" variant="danger" dismissible onClose={() => removeError()}>{ error }</Alert>

  return <React.Fragment>
    <TopNavbar />
    {
      props.container ? 
        <Container className={props.space ? "mt-3" : ""}> { alert } { props.children } </Container> 
        : <React.Fragment> { alert } { props.children } </React.Fragment>
    }
    <BottomNavbar />
  </React.Fragment>
};

const ARENA_STATE_MAP: { [k in ArenaState["state"]]: string } = {
  "Init": "Initialising...",
  "Idle": "Idle",
  "Estop": "EMERGENCY STOP",
  "MatchArmed": "ARMED",
  "MatchComplete": "Match Complete",
  "MatchPlay": "Match Play",
  "Prestart": "Prestart",
  "Reset": "Resetting..."
};

export function TopNavbar() {
  const { user, connected, logout, subscribe, unsubscribe, call } = useWebsocket();
  const { addError } = useErrors();

  const [ arenaState, setArenaState ] = useState<ArenaState>();
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);

  useEffect(() => {
    let handles = [
      subscribe<"arena/state">("arena/state", setArenaState),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
    ];
    return () => unsubscribe(handles);
  }, []);

  let state_comment = arenaState ? ARENA_STATE_MAP[arenaState.state] : "Waiting...";
  if (currentMatch) {
    if (currentMatch.state in ["Auto", "Pause", "Teleop"]) 
      state_comment = `${currentMatch.state} (T-${currentMatch.remaining.toFixed(0)})`;
  }

  return <Navbar className="navbar-top" data-connected={connected} data-arena-state={arenaState?.state} data-match-state={currentMatch?.state} variant="dark">
    <Container>
      <PermissionGate permissions={["Estop"]}>
        { arenaState?.state === "Estop" ? <PermissionGate permissions={["FTA"]}>
          <Button className="mx-3" variant="estop-reset" onClick={() => call<"arena/signal">("arena/signal", { signal: "EstopReset" })}>RESET</Button>
        </PermissionGate> : <Button className="mx-3" variant="estop" onClick={() => estopModal(call, addError)}>E-STOP</Button>}
      </PermissionGate>

      <Navbar.Brand>
        <Link href="/" style={{ textDecoration: "none", color: "white" }}> <strong>JMS</strong> </Link>
        &nbsp; &nbsp;
        <strong style={{ color: "white" }}>{ connected ? state_comment : "[DISCONNECTED]" }</strong>
      </Navbar.Brand>
      <Navbar.Toggle/>
      <Navbar.Collapse className="justify-content-end">
        <Nav>
          {
            user ? <Navbar.Text>
              Hello <span style={{ color: "white" }}>{ user.username }</span> &nbsp;
              <Button size="sm" variant="red" onClick={() => logout()}> <FontAwesomeIcon icon={faRightFromBracket} /> </Button>
            </Navbar.Text> : <Link href="/login"> <Button variant="secondary"> LOGIN </Button> </Link>
          }
        </Nav>
      </Navbar.Collapse>
    </Container>
  </Navbar>;
}

export function BottomNavbar() {
  const { subscribe, unsubscribe, call } = useWebsocket();
  const [ components, setComponents ] = useState<JmsComponent[]>([]);

  useEffect(() => {
    let cbs = [
      subscribe<"components/components">("components/components", setComponents)
    ];
    return () => unsubscribe(cbs);
  }, []);

  return <Row className="navbar-bottom">
    <Col>
      {
        components.sort((a, b) => a.symbol.localeCompare(b.symbol)).map(c => <ComponentIndicator component={c} />)
      }
    </Col>
    <Col md="auto">
      1h 53m BEHIND
    </Col>
    <Col>
    </Col>
  </Row>
}

async function estopModal(call: JmsWebsocket["call"], addError: (e: string) => void) {
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
    call<"arena/signal">("arena/signal", { signal: "Estop" }).catch(addError);
  }
}

function ComponentIndicator({ component }: { component: JmsComponent }) {
  let [ now, setNow ] = useState(moment());

  useEffect(() => {
    const interval = setInterval(() => setNow(moment()), 100);
    return () => clearInterval(interval);
  }, []);

  return <SimpleTooltip id={ component.id } tip={component.name}>
    <div className="jms-component" data-heartbeat={now.diff(moment(component.last_tick)) < component.timeout_ms}>
      { component.symbol }
    </div>
  </SimpleTooltip>
}