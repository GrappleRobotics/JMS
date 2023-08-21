"use client";
import confirmBool from "@/app/components/Confirm";
import "../../estop.scss";
import { useToasts } from "@/app/support/errors";
import { capitalise } from "@/app/support/strings";
import { useWebsocket } from "@/app/support/ws-component";
import { Alliance, AllianceStation, DriverStationReport } from "@/app/ws-schema";
import React from "react";
import { useEffect, useState } from "react";
import { Button, Col, Container, Row } from "react-bootstrap";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCarBattery, faCode, faRobot, faWifi } from "@fortawesome/free-solid-svg-icons";

export default function TeamEstop({ params }: { params: { alliance: Alliance, station: string } }) {
  const [ allianceStation, setAllianceStation ] = useState<AllianceStation>();
  const [ dsReports, setDsReports ] = useState<DriverStationReport[]>([]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

  useEffect(() => {
    let cbs = [
      subscribe<"arena/ds">("arena/ds", setDsReports),
      subscribe<"arena/stations">("arena/stations", stns => setAllianceStation(stns.find(stn => stn.id.alliance === params.alliance && stn.id.station === parseInt(params.station))))
    ];
    return () => unsubscribe(cbs);
  }, []);

  const triggerEstop = async (mode: "estop" | "astop") => {
    const subtitle = <p className="estop-subtitle text-muted">
      Are you sure? The emergency stop is permanent for this { mode === "estop" ? "match" : "autonomous period" } and cannot be reverted by the field crew. <br />
      Robot E-Stops are not eligible for a match replay. <br /> <br />
      <h3 className="text-danger"><strong>THIS WILL DISABLE YOUR ROBOT FOR THE REST OF { mode === "estop" ? "THE MATCH" : "AUTONOMOUS" } </strong></h3>
    </p>
    let result = await confirmBool(subtitle, {
      size: "xl",
      okBtn: {
        size: "lg",
        className: "estop-big",
        variant: mode,
        children: "EMERGENCY STOP"
      },
      cancelBtn: {
        size: "lg",
        className: "btn-block",
        children: "CANCEL",
        variant: "secondary"
      }
    });

    if (result) {
      call<"arena/estop_station">("arena/estop_station", { astop: mode === "astop", station_id: allianceStation!.id }).catch(addError);
    }
  };

  if (!allianceStation)
    return <Container>
      <h3 className="mt-4">Waiting...</h3>
    </Container>
  
  const report = dsReports.find(ds => ds.team === allianceStation.team);

  return <div className="team-estop">
    <h3> { capitalise(allianceStation.id.alliance) } { allianceStation.id.station } - { allianceStation.team || "No Team" } </h3>
    <br />
    <Row className="team-estop-indicators">
      <Col data-ok={report?.radio_ping}>
        <FontAwesomeIcon icon={faWifi} /> &nbsp;
        { report?.radio_ping ? 
          `${(report?.rtt?.toString() || "---").padStart(3, "\u00A0")}ms`
          : "NO RADIO"
        }
      </Col>
      <Col data-ok={report?.rio_ping}>
        <FontAwesomeIcon icon={faRobot} /> &nbsp;
        { report?.rio_ping ? "RIO OK" : "NO RIO" }
      </Col>
      <Col data-ok={report?.robot_ping}>
        <FontAwesomeIcon icon={faCode} /> &nbsp;
        { report?.robot_ping ? "CODE OK" : "NO CODE" }
      </Col>
      <Col data-ok={report?.battery_voltage || 0 > 10}>
        <FontAwesomeIcon icon={faCarBattery} /> &nbsp;
        { report?.battery_voltage?.toFixed(2) || "--.--" } V
      </Col>
      <Col data-estop={report?.estop}>
        { report?.estop ? "ROBOT ESTOP" : (report?.mode || "---").toUpperCase() }
      </Col>
    </Row>
    <br />
      <Button
        size="lg"
        className="estop-all"
        variant={ allianceStation.estop ? "secondary" : "estop" } 
        disabled={allianceStation.estop}
        onClick={() => triggerEstop("estop")}
      >
        EMERGENCY STOP <br />
        <span className="subtext"> AUTO + TELEOP </span>
      </Button>
      <br />
      <Button
        className="estop-auto"
        variant={ (allianceStation.estop || allianceStation.astop) ? "secondary" : "hazard-yellow" }
        disabled={allianceStation.astop || allianceStation.estop}
        onClick={() => triggerEstop("astop")}
      >
        EMERGENCY STOP <br />
        <span className="subtext"> AUTO ONLY</span>
      </Button>
  </div>
}