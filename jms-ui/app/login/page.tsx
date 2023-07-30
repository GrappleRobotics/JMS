"use client";
import { WebsocketComponent } from "../support/ws-component";
import UserPage from "../userpage";
import { useRouter } from "next/navigation";
import { AppRouterInstance } from "next/dist/shared/lib/app-router-context";
import React from "react";
import { Alert, Button, Card, Form, InputGroup } from "react-bootstrap";
import "./login.scss";

interface LoginPageState {
  username: string,
  pin: string,
  error?: string
};

export default (props: LoginPageState) => {
  let router = useRouter();
  return <LoginPageInner { ...props } router={router} />
}

class LoginPageInner extends WebsocketComponent<{ router: AppRouterInstance }, LoginPageState> {
  readonly state: LoginPageState = {
    username: "",
    pin: ""
  }

  render() {
    return <UserPage container space>
      {
        this.user() && <Alert variant="info">You're already logged in!</Alert>
      }
      <Card className="login-card">
        <Card.Body>
          <h5>Login to JMS</h5>
          <InputGroup className="mt-3 mb-2">
            <InputGroup.Text>Username</InputGroup.Text>
            <Form.Control
              type="text"
              value={this.state.username}
              onChange={e => this.setState({ username: e.target.value })}
              isInvalid={!!this.state.error}
            />
          </InputGroup>
          <InputGroup className="mt-3 mb-2">
            <InputGroup.Text>PIN</InputGroup.Text>
            <Form.Control
              type="password"
              value={this.state.pin}
              onChange={e => this.setState({ pin: e.target.value })}
              isInvalid={!!this.state.error}
            />
          </InputGroup>
          <Button
            variant="primary"
            size="lg"
            style={{ width: '100%' }}
            onClick={() => {
              this.login(this.state.username, this.state.pin)
                .then(() => this.props.router.push("/"))
                .catch(e => this.setState({ error: e }))
            }}
          >
            LOGIN
          </Button>
        </Card.Body>
      </Card>
    </UserPage>
  }
}