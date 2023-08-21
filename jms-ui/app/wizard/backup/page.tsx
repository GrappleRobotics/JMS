"use client"

import { useToasts } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { BackupSettings, BackupSettingsUpdate, JmsComponent } from "@/app/ws-schema";
import React, { useEffect, useState } from "react";
import { Alert, Button, Col, Form, InputGroup, Row } from "react-bootstrap";
import { saveAs } from 'file-saver';
import moment from "moment";
import { withConfirm } from "@/app/components/Confirm";
import BufferedFormControl from "@/app/components/BufferedFormControl";
import { nullIfEmpty } from "@/app/support/strings";

export default withPermission(["FTA"], function EventWizardBackups() {
  const [ settings, setSettings ] = useState<BackupSettings>();
  const [ components, setComponents ] = useState<[string, JmsComponent[]]>(["", []]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

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
    <p className="text-danger">WARNING: Backups contain raw TheBlueAlliance and S3 credentials. Keep your backups safe and away from prying eyes.</p>

    {
      !components[1].find(c => c.id === "jms.backup") && <Alert variant="danger">
        <h4> JMS-Backup module is not loaded! </h4>
        In order to use the Backup feature, you need to start the JMS-Backup container.
      </Alert>
    }

    { settings && <React.Fragment>
      <h4 className="mt-2">Filesystem Target</h4>
      <Row className="mt-2">
        <Col>
          <InputGroup>
            <InputGroup.Text>Target Directory</InputGroup.Text>
            <BufferedFormControl
              type="text"
              value={settings.file_target_dir || ""}
              onUpdate={v => update({ file_target_dir: nullIfEmpty(v as string) })}
            />
          </InputGroup>
        </Col>
      </Row>

      <h4 className="mt-4">S3 Target</h4>
      <Row className="mt-2">
        <Col>
          <InputGroup>
            <InputGroup.Text>S3 Endpoint</InputGroup.Text>
            <BufferedFormControl
              type="text"
              value={settings.s3_endpoint}
              onUpdate={v => update({ s3_endpoint: v as string })}
            />
          </InputGroup>
        </Col>
        <Col>
          <InputGroup>
            <InputGroup.Text>S3 Region</InputGroup.Text>
            <BufferedFormControl
              type="text"
              value={settings.s3_region}
              onUpdate={v => update({ s3_region: v as string })}
            />
          </InputGroup>
        </Col>
      </Row>
      <Row className="mt-2">
        <Col>
          <InputGroup>
            <InputGroup.Text>S3 Access Key</InputGroup.Text>
            <BufferedFormControl
              type="text"
              value={settings.s3_access_key || ""}
              onUpdate={v => update({ s3_access_key: nullIfEmpty(v as string) })}
            />
          </InputGroup>
        </Col>
        <Col>
          <InputGroup>
            <InputGroup.Text>S3 Secret Access Key</InputGroup.Text>
            <BufferedFormControl
              type="password"
              value={settings.s3_secret_access_key || ""}
              onUpdate={v => update({ s3_secret_access_key: nullIfEmpty(v as string) })}
            />
          </InputGroup>
        </Col>
      </Row>
      <Row className="mt-2">
        <Col>
          <InputGroup>
            <InputGroup.Text>S3 Bucket</InputGroup.Text>
            <BufferedFormControl
              type="text"
              value={settings.s3_bucket || ""}
              onUpdate={v => update({ s3_bucket: nullIfEmpty(v as string) })}
            />
          </InputGroup>
        </Col>
      </Row>

      <h4 className="mt-4">Backup & Restore</h4>
      <Row className="mt-2">
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