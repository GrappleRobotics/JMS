"use client"

import BufferedFormControl from "@/app/components/BufferedFormControl";
import { useToasts } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { ScoringConfig } from "@/app/ws-schema";
import update, { Spec } from "immutability-helper";
import { useEffect, useState } from "react";
import { Col, Container, Row } from "react-bootstrap";

export default withPermission(["EditScores"], function EditScoringConfig() {
  const [ config, setConfig ] = useState<ScoringConfig | null>(null);
  
  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

  useEffect(() => {
    let cbs = [
      subscribe<"scoring/config">("scoring/config", s => setConfig(s)),
    ];
    return () => unsubscribe(cbs);
  }, []);

  const updateConfig = (u: Spec<ScoringConfig>) => call<"scoring/update_config">("scoring/update_config", { config: update(config!, u) })

  return <Container>
    <Row>
      <Col>
        <h2 className="mt-3 mb-0"> Update Scoring Config </h2>
      </Col>
    </Row>
    <Row>
      { !config ? <p> Waiting... </p> : <Col>
        {
          Object.keys(config).map(key => <Row key={key}>
            <Col md={3}> { key } </Col>
            <Col>
              <BufferedFormControl
                type="number"
                min={0}
                step={1}
                value={(config as any)[key] as number}
                onUpdate={v => updateConfig({ [key]: { $set: (v as number) || 0 } })}
                auto
                enter
              />
            </Col>
          </Row>)
        }
      </Col> }
    </Row>
  </Container>
});