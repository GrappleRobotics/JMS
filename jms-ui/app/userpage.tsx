"use client";
import { useWebsocket } from "./support/ws-component";
import React, { useEffect, useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faRightFromBracket } from "@fortawesome/free-solid-svg-icons";
import { Alert, Button, Container, Nav, Navbar } from "react-bootstrap";
import "./userpage.scss";
import Link from "next/link";
import { useErrors } from "./support/errors";
import { ArenaState, SerialisedLoadedMatch } from "./ws-schema";
import { PermissionGate } from "./support/permissions";

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
  </React.Fragment>
};

function TopNavbar() {
  const { user, connected, logout, subscribe, unsubscribe } = useWebsocket();

  const [ arenaState, setArenaState ] = useState<ArenaState>();
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);

  useEffect(() => {
    let handles = [
      subscribe<"arena/state">("arena/state", setArenaState),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
    ];
    return () => unsubscribe(handles);
  }, []);

  let state_comment = arenaState?.state || "Waiting...";
  if (currentMatch) {
    if (currentMatch.state in ["Auto", "Pause", "Teleop"]) 
      state_comment = `${currentMatch.state} (T-${currentMatch.remaining.toFixed(0)})`;
  }

  return <Navbar className="navbar-top" data-connected={connected} data-arena-state={arenaState?.state} data-match-state={currentMatch?.state} variant="dark">
    <Container>
      <PermissionGate permissions={["Estop"]}>
        <Button className="mx-3" variant="estop">E-STOP</Button>
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
