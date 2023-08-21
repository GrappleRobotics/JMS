"use client"
import BufferedFormControl from "@/app/components/BufferedFormControl";
import { useToasts } from "@/app/support/errors";
import { nullIfEmpty } from "@/app/support/strings";
import { useWebsocket } from "@/app/support/ws-component";
import { JmsComponent, TBASettings } from "@/app/ws-schema";
import { faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React, { useEffect, useState } from "react";
import { Alert, Col, InputGroup, Row } from "react-bootstrap";

export default function TBAIntegration() {
  const [ settings, setSettings ] = useState<TBASettings>();
  const [ components, setComponents ] = useState<[string, JmsComponent[]]>(["", []]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

  useEffect(() => {
    call<"tba/get_settings">("tba/get_settings", null)
      .then(setSettings)
      .catch(addError);
    
    let cbs = [
      subscribe<"components/components">("components/components", setComponents)
    ];
    return () => unsubscribe(cbs);
  }, []);

  return <React.Fragment>
    <h3> The Blue Alliance </h3>
    <p className="text-muted"> <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; To utilise The Blue Alliance integrations, it is required
    that your offseason is listen on TBA, and that you have API write keys, which should be requested prior to your event. You can request
    API keys <a href="https://www.thebluealliance.com/request/apiwrite/" target="_blank">here</a>. </p>

    {
      !components[1].find(c => c.id === "jms.tba") && <Alert variant="danger">
        <h4> JMS-TBA module is not loaded! </h4>
        In order to use The Blue Alliance integrations, you need to start the JMS-TBA container.
      </Alert>
    }

    { settings && <Row>
      <Col>
        <InputGroup>
          <InputGroup.Text>Auth ID</InputGroup.Text>
          <BufferedFormControl
            type="text"
            value={settings.auth_id || ""}
            onUpdate={v => call<"tba/update_settings">("tba/update_settings", { update: { auth_id: nullIfEmpty(v as string) } })}
          />
        </InputGroup>

        <InputGroup className="mt-2">
          <InputGroup.Text>Auth Secret ID (Auth Key)</InputGroup.Text>
          <BufferedFormControl
            type="text"
            value={settings.auth_key || ""}
            onUpdate={v => call<"tba/update_settings">("tba/update_settings", { update: { auth_key: nullIfEmpty(v as string) } })}
          />
        </InputGroup>
      </Col>
    </Row> }
  </React.Fragment>
}