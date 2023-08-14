"use client"

import BufferedFormControl from "@/app/components/BufferedFormControl";
import EnumToggleGroup from "@/app/components/EnumToggleGroup";
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { nullIfEmpty } from "@/app/support/strings";
import { useWebsocket } from "@/app/support/ws-component";
import { JmsComponent, NetworkingSettings, NetworkingSettingsUpdate, RadioType } from "@/app/ws-schema";
import React from "react";
import { useEffect, useState } from "react";
import { Alert, Button, Col, Form, InputGroup, Row } from "react-bootstrap";

export default withPermission(["FTA"], function AdvancedNetworking() {
  const [ settings, setSettings ] = useState<NetworkingSettings>();
  const [ components, setComponents ] = useState<[string, JmsComponent[]]>(["", []]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    call<"networking/settings">("networking/settings", null)
      .then(setSettings)
      .catch(addError);
    
    let cbs = [
      subscribe<"components/components">("components/components", setComponents)
    ];
    return () => unsubscribe(cbs);
  }, []);

  const update = (update: NetworkingSettingsUpdate) => {
    call<"networking/update_settings">("networking/update_settings", { update }).then(setSettings).catch(addError);
  }

  return <React.Fragment>
    <h3> Advanced Networking </h3>

    {
      !components[1].find(c => c.id === "jms.networking") && <Alert variant="danger">
        <h4> JMS-Networking module is not loaded! </h4>
        In order to use Advanced Networking, you need to start the JMS-Networking container and the corresponding PFSense VM.
      </Alert>
    }

    { settings && <Row>
      <Col>
        <h4 className="mt-3"> Router Settings </h4>
        <Row className="mt-2">
          <Col>
            <InputGroup>
              <InputGroup.Text>Username</InputGroup.Text>
              <BufferedFormControl
                type="text"
                value={settings.router_username}
                onUpdate={v => update({ router_username: v as string })}
              />
            </InputGroup>
          </Col>
          <Col>
            <InputGroup>
              <InputGroup.Text>Password</InputGroup.Text>
              <BufferedFormControl
                type="password"
                value={settings.router_password}
                onUpdate={v => update({ router_password: v as string })}
              />
            </InputGroup>
          </Col>
        </Row>

        <h4 className="mt-3"> Radio Settings </h4>
        <Row className="mt-2">
          <Col>
            <InputGroup>
              <InputGroup.Text>Radio Type</InputGroup.Text> &nbsp;
              <EnumToggleGroup
                name="radio-type"
                value={settings.radio_type}
                values={[ "Linksys", "Unifi" ] as RadioType[]}
                onChange={v => update({ radio_type: v })}
                variant="secondary"
              />
            </InputGroup>
          </Col>
        </Row>

        <Row className="mt-2">
          <Col>
            <InputGroup>
              <InputGroup.Text>Username</InputGroup.Text>
              <BufferedFormControl
                type="text"
                value={settings.radio_username}
                onUpdate={v => update({ radio_username: v as string })}
              />
            </InputGroup>
          </Col>
          <Col>
            <InputGroup>
              <InputGroup.Text>Password</InputGroup.Text>
              <BufferedFormControl
                type="password"
                value={settings.radio_password}
                onUpdate={v => update({ radio_password: v as string })}
              />
            </InputGroup>
          </Col>
        </Row>

        <Row className="mt-2">
          <Col>
            <InputGroup>
              <InputGroup.Text>Admin SSID</InputGroup.Text>
              <BufferedFormControl
                type="text"
                value={settings.admin_ssid || ""}
                disabled={settings.radio_type === "Unifi"}
                onUpdate={v => update({ admin_ssid: nullIfEmpty(v as string) })}
              />
            </InputGroup>
          </Col>
          <Col>
            <InputGroup>
              <InputGroup.Text>Admin Passkey</InputGroup.Text>
              <BufferedFormControl
                type="text"
                value={settings.admin_password || ""}
                disabled={settings.radio_type === "Unifi"}
                onUpdate={v => update({ admin_password: nullIfEmpty(v as string) })}
              />
            </InputGroup>
          </Col>
        </Row>

        <Row className="mt-2">
          <Col md="auto">
            <InputGroup>
              <InputGroup.Text>Team Channel</InputGroup.Text>
              <Form.Select value={settings.team_channel || "auto"} onChange={v => update({ team_channel: v.target.value === "auto" ? null : parseInt(v.target.value) })}>
                {
                  ["auto", 32, 36, 40, 44, 48, 52, 56, 60, 64, 68, 96, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 144, 149, 153, 157, 161, 165, 169, 173, 177]
                    .map(channel => <option key={channel} value={channel}>{ channel }</option>)
                }
              </Form.Select>
            </InputGroup>
          </Col>

          <Col md="auto">
            <InputGroup>
              <InputGroup.Text>Admin Channel</InputGroup.Text>
              <Form.Select disabled={settings.radio_type === "Unifi"} value={settings.admin_channel || "auto"} onChange={v => update({ admin_channel: v.target.value === "auto" ? null : parseInt(v.target.value) })}>
                {
                  ["auto", 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]
                    .map(channel => <option key={channel} value={channel}>{ channel }</option>)
                }
              </Form.Select>
            </InputGroup>
          </Col>
        </Row>

        <Row className="mt-2">
          <Col>
            <Button size="lg" variant="success" disabled={settings.radio_type !== "Linksys"} onClick={() => call<"networking/reload_admin">("networking/reload_admin", null).catch(addError)}>
              Reload Admin Network (Linksys Only)
            </Button> &nbsp;
            <Button size="lg" variant="danger" disabled={settings.radio_type !== "Unifi"} onClick={() => call<"networking/force_reprovision">("networking/force_reprovision", null).catch(addError)}>
              Force Reprovision (Unifi Only)
            </Button>
          </Col>
        </Row>
      </Col>
    </Row>}
  </React.Fragment>
});