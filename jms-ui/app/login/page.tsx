"use client";
import { useWebsocket } from "../support/ws-component";
import UserPage from "../userpage";
import { useRouter } from "next/navigation";
import { AppRouterInstance } from "next/dist/shared/lib/app-router-context";
import React, { useState } from "react";
import { Alert, Button, Card, Form, InputGroup } from "react-bootstrap";
import "./login.scss";
import { useToasts } from "../support/errors";

export default function LoginPage() {
  const router = useRouter();
  const { user, login } = useWebsocket();
  const errorCtx = useToasts();

  const [username, setUsername] = useState("");
  const [pin, setPin] = useState("");
  const [error, setError] = useState<string | undefined>(undefined);

  return <UserPage container space>
    {
      user && <Alert variant="info">{ "You're already logged in! "}</Alert>
    }
    <Card className="login-card">
      <Card.Body>
        <h5>Login to JMS</h5>
        <InputGroup className="mt-3 mb-2">
          <InputGroup.Text>Username</InputGroup.Text>
          <Form.Control
            type="text"
            value={username}
            onChange={e => setUsername(e.target.value)}
            isInvalid={!!error}
          />
        </InputGroup>
        <InputGroup className="mt-3 mb-2">
          <InputGroup.Text>PIN</InputGroup.Text>
          <Form.Control
            type="password"
            value={pin}
            onChange={e => setPin(e.target.value)}
            isInvalid={!!error}
          />
        </InputGroup>
        <Button
          variant="primary"
          size="lg"
          style={{ width: '100%' }}
          onClick={() => {
            login(username, pin)
              .then(() => { router.push("/"); errorCtx.removeError() })
              .catch(e => { setError(e); errorCtx.addError(e as string); })
          }}
        >
          LOGIN
        </Button>
      </Card.Body>
    </Card>
  </UserPage>
}