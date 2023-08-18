"use client"

import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { BackupSettings, BackupSettingsUpdate, JmsComponent } from "@/app/ws-schema";
import React, { useEffect, useState } from "react";
import { Alert, Button, Col, Form, Row } from "react-bootstrap";
import { saveAs } from 'file-saver';
import moment from "moment";
import { withConfirm } from "@/app/components/Confirm";

export default withPermission(["FTA"], function EventWizardBackups() {
  const [ settings, setSettings ] = useState<BackupSettings>();
  const [ components, setComponents ] = useState<[string, JmsComponent[]]>(["", []]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    call<"backup/settings">("backup/settings", null)
      .then(setSettings)
      .catch(addError);
    
    let cbs = [
      subscribe<"components/components">("components/components", setComponents)
    ];
    return () => unsubscribe(cbs);
  }, []);

  const update = (update: BackupSettingsUpdate) => {
    call<"backup/update_settings">("backup/update_settings", { update }).then(setSettings).catch(addError);
  }

  return <React.Fragment>
    <h3> Backups </h3>

    {
      !components[1].find(c => c.id === "jms.backup") && <Alert variant="danger">
        <h4> JMS-Backup module is not loaded! </h4>
        In order to use the Backup feature, you need to start the JMS-Backup container.
      </Alert>
    }

    { settings && <React.Fragment>
      <Row>
        <Col md="auto">
          <Button size="lg" variant="success" onClick={() => call<"backup/backup_now">("backup/backup_now", null).catch(addError)}>
            Backup Now
          </Button> &nbsp;
          <Button size="lg" variant="primary" onClick={() => call<"backup/backup_to">("backup/backup_to", null).then(data => saveAs(new Blob([data], { type: "application/json" }), `jms-backup-${moment().toISOString()}.json`)).catch(addError)}>
            Backup to File
          </Button> &nbsp;
        </Col>
        <Col>
          <Form.Control
            type="file"
            onChange={e => {
              // @ts-ignore
              if (e.target.files.length > 0) {
                withConfirm(() => {
                  // @ts-ignore
                  let file = e.target.files[0];
                  let reader = new FileReader();
                  reader.onload = event => {
                    if (event.target?.result) {
                      call<"backup/restore">("backup/restore", { data: event.target.result as string })
                        .then(() => alert("Backup Restored!"))
                        .catch(addError)
                    }
                  };
                  reader.readAsText(file);
                })
              }
            }}
          />
          <Form.Text>Restore from File</Form.Text>
        </Col>
      </Row>
    </React.Fragment>}
  </React.Fragment>
})