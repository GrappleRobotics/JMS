"use client"

import EnumToggleGroup from "@/app/components/EnumToggleGroup";
import { useToasts } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions"
import { useWebsocket } from "@/app/support/ws-component";
import { EstopMode, FieldElectronicsSettings, FieldElectronicsSettingsUpdate, JmsComponent } from "@/app/ws-schema";
import React, { useEffect, useState } from "react"
import { Alert, Button, Col, InputGroup, Row } from "react-bootstrap";

export default withPermission(["ManageElectronics"], function Electronics() {
  const [ settings, setSettings ] = useState<FieldElectronicsSettings>();
  const [ components, setComponents ] = useState<[string, JmsComponent[]]>(["", []]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

  useEffect(() => {
    call<"electronics/settings">("electronics/settings", null)
      .then(setSettings)
      .catch(addError);
    
    let cbs = [
      subscribe<"components/components">("components/components", setComponents)
    ];
    return () => unsubscribe(cbs);
  }, []);

  const update = (update: FieldElectronicsSettingsUpdate) => {
    call<"electronics/update_settings">("electronics/update_settings", { update }).then(setSettings).catch(addError);
  }

  return <React.Fragment>
    <h3> Field Electronics </h3>

    {
      !components[1].find(c => c.id === "jms.electronics") && <Alert variant="danger">
        <h4> JMS-Electronics module is not loaded! </h4>
        In order to use Field Electronics, you need to start the JMS-Electronics container.
      </Alert>
    }

    {
      settings && <Row>
        <Col>
          <h4 className="mt-3"> Electronics Settings </h4>
          <InputGroup>
            <InputGroup.Text>E-Stop Mode</InputGroup.Text> &nbsp;
            <EnumToggleGroup
              name="estop-mode"
              value={settings.estop_mode}
              values={[ "NormallyClosed", "NormallyOpen" ] as EstopMode[]}
              names={[ "Normally Closed", "Normally Open" ]}
              onChange={v => update({ estop_mode: v })}
              variant="secondary"
            />
          </InputGroup>
        </Col>
        <Col md="4">
          <Button variant="danger" onClick={() => call<"electronics/reset_estops">("electronics/reset_estops", null).catch(addError)}>
            Reset Estops
          </Button>
        </Col>
      </Row>
    }
  </React.Fragment>
})