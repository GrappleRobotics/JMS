"use client";
import { WebsocketComponent } from "./support/ws-component";
import React from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faRightFromBracket } from "@fortawesome/free-solid-svg-icons";
import { Button, Container, Nav, Navbar } from "react-bootstrap";
import "./userpage.scss";
import Link from "next/link";

interface UserPageProps {
  container?: boolean,
  space?: boolean,
  children: React.ReactNode
};

// TODO: Move back to react-bootstrap, I don't like how MUI does all the styling in JS.

export default class UserPage extends WebsocketComponent<UserPageProps> {
  render() {
    return <React.Fragment>
      <TopNavbar />
      {
        this.props.container ? <Container className={this.props.space ? "mt-3" : ""}> { this.props.children } </Container> : this.props.children
      }
    </React.Fragment>
  }
}

class TopNavbar extends WebsocketComponent {
  render() {
    return <Navbar className="navbar-top" data-connected={this.isConnected()} variant="dark">
      <Container>
        <Button className="mx-3" variant="estop">E-STOP</Button>
        <Navbar.Brand>
          <Link href="/" style={{ textDecoration: "none", color: "white" }}> <strong>JMS</strong> </Link>
        </Navbar.Brand>
        // &nbsp;
        <Navbar.Brand>
          <strong style={{ color: "white" }}>{ this.isConnected() ? "" : "[DISCONNECTED]" }</strong>
        </Navbar.Brand>
        <Navbar.Toggle/>
        <Navbar.Collapse className="justify-content-end">
          <Nav>
            {
              this.user() ? <Navbar.Text>
                Hello <span style={{ color: "white" }}>{ this.user()?.username }</span> &nbsp;
                <Button size="sm" variant="red" onClick={() => this.logout()}> <FontAwesomeIcon icon={faRightFromBracket} /> </Button>
              </Navbar.Text> : <Link href="/login"> <Button variant="secondary"> LOGIN </Button> </Link>
            }
          </Nav>
        </Navbar.Collapse>
      </Container>
    </Navbar>
  }
}