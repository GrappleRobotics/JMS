import { faCarBattery, faCode, faRobot, faWifi } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import confirmBool from "components/elements/Confirm";
import { FieldResourceSelector } from "components/FieldPosSelector";
import React from "react";
import { Button, Col, Row } from "react-bootstrap";
import { Link, Route, Routes } from "react-router-dom";
import { capitalise } from "support/strings";
import { withValU } from "support/util";
import { WebsocketComponent, withRole } from "support/ws-component";
import { AllianceStationDSReport, AllianceStationOccupancy, SerialisedAllianceStation } from "ws-schema";

type TeamEstopProps = {
  station: SerialisedAllianceStation,
  onTrigger: (which: "astop" | "estop") => void
}

class TeamEstop extends React.PureComponent<TeamEstopProps> {  
  triggerEstop = async (mode: "estop" | "astop") => {
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
      this.props.onTrigger(mode);
    }
  }

  renderDsReport = (eth: boolean, occupancy: AllianceStationOccupancy, report?: AllianceStationDSReport | null) => {
    const occupancy_str = {
      "Vacant": eth ? "Ethernet Connected, Driver Station Not Connected" : "No Driver Station Connected",
      "WrongMatch": "Wrong Team Number - Wrong Match?",
      "WrongStation": "Wrong Station - Move!"
    };

    return <Row className="team-estop-indicators">
      {
        occupancy !== "Occupied" ? <Col data-ok={false}> { occupancy_str[occupancy] } </Col>
        : <React.Fragment>
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
            <Col data-ok={report?.battery || 0 > 10}>
              <FontAwesomeIcon icon={faCarBattery} /> &nbsp;
              { report?.battery?.toFixed(2) || "--.--" } V
            </Col>
            <Col data-estop={report?.estop}>
              { report?.estop ? "ROBOT ESTOP" : (report?.mode || "---").toUpperCase() }
            </Col>
        </React.Fragment>
      }
    </Row>
  }

  render() {
    let { station } = this.props;
    return <div className="team-estop">
      <h3> { capitalise(station.station.alliance) } { station.station.station } - { station.team || "No Team" } </h3>
      <br />
      { this.renderDsReport(station.ds_eth, station.occupancy, station.ds_report) }
      <br />
      <Button
        size="lg"
        className="estop-all"
        variant={ station.estop ? "secondary" : "estop" } 
        disabled={station.estop}
        onClick={() => this.triggerEstop("estop")}
      >
        EMERGENCY STOP <br />
        <span className="subtext"> AUTO + TELEOP </span>
      </Button>
      <br />
      <Button
        className="estop-auto"
        variant={ (station.estop || station.astop) ? "secondary" : "hazard-yellow" }
        disabled={station.astop || station.estop}
        onClick={() => this.triggerEstop("astop")}
      >
        EMERGENCY STOP <br />
        <span className="subtext"> AUTO ONLY</span>
      </Button>
    </div>
  }
}

type TeamEstopsState = {
  stations: SerialisedAllianceStation[]
};

export class TeamEstops extends WebsocketComponent<{}, TeamEstopsState> {
  readonly state: TeamEstopsState = { stations: [] };

  componentDidMount = () => this.handles = [
    this.listen("Arena/Alliance/CurrentStations", "stations")
  ]

  selector = () => {
    const { stations } = this.state;
    return <FieldResourceSelector
      title="Select Team"
      options={ stations.map(s => ( { TeamEStop: s.station } )) }
      labels={ stations.map(s => (
        `${capitalise(s.station.alliance)} ${s.station.station} ${ s.team ? ` - ${s.team}` : "" }`
      )) }
      wrap={(r, child) => <Link to={`${r.TeamEStop.alliance}-${r.TeamEStop.station}`}> { child } </Link>}
    />
  }

  render() {
    return <Routes>
      <Route path="/" element={ this.selector() }/>

      {
      this.state.stations.map((s, i) => (
        <Route key={i} path={`${s.station.alliance}-${s.station.station}`} element={
          withRole({ TeamEStop: s.station }, <TeamEstop
            station={s}
            onTrigger={which => this.send({ Arena: { Alliance: { UpdateAlliance: { [which]: true, station: s.station } } } })}
          />)
        }/>
        ))
      }
    </Routes>
  }
}