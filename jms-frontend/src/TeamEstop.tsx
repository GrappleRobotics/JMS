import confirmBool from "components/elements/Confirm";
import { TeamSelector } from "components/FieldPosSelector";
import React from "react";
import { Button, Col, Container } from "react-bootstrap";
import { Link, Navigate, Route, Routes, useNavigate } from "react-router-dom";
import { capitalise } from "support/strings";
import { WebsocketComponent, withRole } from "support/ws-component";
import { AllianceStation } from "ws-schema";

type TeamEstopProps = {
  station: AllianceStation,
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

  render() {
    let { station } = this.props;
    return <div className="team-estop">
      <h3> { capitalise(station.station.alliance) } { station.station.station } - { station.team || "No Team" } </h3>
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
  stations: AllianceStation[]
};

export class TeamEstops extends WebsocketComponent<{}, TeamEstopsState> {
  readonly state: TeamEstopsState = { stations: [] };

  componentDidMount = () => this.handles = [
    this.listen("Arena/Alliance/CurrentStations", "stations")
  ]

  render() {
    return <Routes>
      <Route path="/" element={ <TeamSelector stations={this.state.stations} /> }/>

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