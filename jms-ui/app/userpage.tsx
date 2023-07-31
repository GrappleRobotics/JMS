"use client";
import { useWebsocket } from "./support/ws-component";
import React from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faRightFromBracket } from "@fortawesome/free-solid-svg-icons";
import { Alert, Button, Container, Nav, Navbar } from "react-bootstrap";
import "./userpage.scss";
import Link from "next/link";
import { useErrors } from "./support/errors";

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
  const { user, connected, logout } = useWebsocket();

  return <Navbar className="navbar-top" data-connected={connected} variant="dark">
    <Container>
      <Button className="mx-3" variant="estop">E-STOP</Button>
      <Navbar.Brand>
        <Link href="/" style={{ textDecoration: "none", color: "white" }}> <strong>JMS</strong> </Link>
      </Navbar.Brand>
      // &nbsp;
      <Navbar.Brand>
        <strong style={{ color: "white" }}>{ connected ? "" : "[DISCONNECTED]" }</strong>
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
