"use client"
import EnumToggleGroup from "@/app/components/EnumToggleGroup";
import SimpleTooltip from "@/app/components/SimpleTooltip";
import { useToasts } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { FieldElectronicsEndpoint, JMSRole } from "@/app/ws-schema";
import { useEffect, useState } from "react";
import { Accordion, Card, Col, Container, Row } from "react-bootstrap";

export default withPermission(["ManageElectronics"], function FieldTest() {
  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();
  
  const [ endpoints, setEndpoints ] = useState<FieldElectronicsEndpoint[]>([]);

  useEffect(() => {
    let cb = [ subscribe<"electronics/endpoints">("electronics/endpoints", setEndpoints) ];
    return () => unsubscribe(cb);
  }, []);

  return <Container className="mt-3">
    <h3> Field Electronics - Field Test </h3>
    <br />
    {
      endpoints.map(ep => <Accordion key={`ep-${ep.ip}`} defaultActiveKey="0" className="mt-3">
        <Accordion.Item eventKey="0">
          <Accordion.Header> <h4>{ ep.ip } - { JSON.stringify(ep.status.role) }</h4> </Accordion.Header>
          <Accordion.Body>
            <Row>
              {
                ep.status.cards.map((card, i) => <Col key={`card-${i}`}>
                  <Card>
                    <Card.Header>Card { i }</Card.Header>
                    <Card.Body>
                      { card != "Lighting" && "IO" in card && card.IO.map((status, j) => {
                        let inner = <span className={status ? "text-good" : "text-bad"} style={{ fontSize: "2em", fontWeight: "bold" }}> { j } </span>;
                        return <span key={`idx-${i}-${j}`}>{ inner }</span>;
                      }) }
                    </Card.Body>
                  </Card>
                </Col>)
              }
            </Row>
            <Row className="mt-3">
              <Col>
                <EnumToggleGroup
                  name="role"
                  value={ep.status.role}
                  values={[ "ScoringTable", { Red: 1 }, { Red: 2 }, { Red: 3 }, { Blue: 1 }, { Blue: 2 }, { Blue: 3 }, "TimerBlue", "TimerRed" ] as JMSRole[]}
                  names={[ "Scoring Table", "Red 1", "Red 2", "Red 3", "Blue 1", "Blue 2", "Blue 3", "Timer (Blue)", "Timer (Red)" ]}
                  onChange={v => call<"electronics/update">("electronics/update", { update: { SetRole: { ip: ep.ip, role: v } } })}
                  variant="secondary"
                />
              </Col>
            </Row>
          </Accordion.Body>
        </Accordion.Item>
      </Accordion>)
    }
  </Container>
})