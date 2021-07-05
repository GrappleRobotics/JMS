import React from 'react';
import { Row, Col, Button, FormControl } from 'react-bootstrap';
import BufferedFormControl from './elements/buffered_control';
 
class Indicator extends React.PureComponent {
  render() {
    return <Button className="indicator-small" disabled size="sm" {...this.props}> 
      { this.props.children || <React.Fragment>&nbsp;</React.Fragment> }
    </Button>
  }
}

class AllianceStation extends React.Component {
  dsStatus = () => {
    switch (this.props.station.occupancy) {
      case "Vacant":
        return <Indicator variant="dark" />;
      case "Occupied":
        return <Indicator variant="success" />;
      case "WrongMatch":
        return <Indicator variant="danger"> W </Indicator>;
      case "WrongStation":
        return <Indicator variant="warning"> M </Indicator>;
    }
  }
  
  render() {
    let s = this.props.station;
    let cname = ["alliance-station"];
    let can_bypass = this.props.state?.state === "Prestart" || this.props.state?.state === "Idle";
    let can_change_team = this.props.state?.state === "Idle";

    if (s.astop)
      cname.push("bg-hazard-dark-active");
    if (s.estop)
      cname.push("bg-hazard-red-dark-active");
    if (s.bypass)
      cname.push("bypassed");

    return <Row className={cname.join(" ")} {...this.props}>
      <Col sm="1">
        <i>{ s.station.station }</i>
      </Col>
      <Col sm="1" className="p-0">
        <Button
          size="sm"
          variant={s.bypass ? "success" : (can_bypass ? "danger" : "dark")}
          disabled={ !can_bypass }
          onClick={() => this.props.onUpdate({ bypass: !s.bypass })}
        >
            BY
        </Button>
      </Col>
      <Col sm="2">
        {/* { s.team || "-" } */}
        <BufferedFormControl
          className="team-num"
          type="number"
          onUpdate={(n) => this.props.onUpdate({ team: parseInt(n) })}
          value={s.team}
          disabled={ !can_change_team }
          placeholder={"----"}
        />
      </Col>
      <Col sm="1">
        {
          this.dsStatus()
        }
      </Col>
      <Col sm="2">
        {
          s.ds_report ?
            <Indicator variant={s.ds_report.robot_ping ? "success" : "danger"} /> :
            <Indicator variant="dark" />
        }

        {
          s.ds_report ? 
            <Indicator variant={s.ds_report.rio_ping ? "success" : "danger"} /> : 
            <Indicator variant="dark" />
        }
      </Col>
      &nbsp;
      <Col>
        {
          (s.ds_report?.battery?.toFixed(1) || "--.-").padStart(4, "\u00A0") + " V"
        }
      </Col>
      <Col>
        {
          (s.ds_report?.rtt?.toString() || "-").padStart(3, "\u00A0")
        }
      </Col>
    </Row>
  }
}

export default class Alliance extends React.Component {
  render() {
    return <div className={"p-3 alliance-" + this.props.colour.toLowerCase()}>
      <Row>
        <Col>
          <Row className="alliance-header">
            <Col sm="1"> # </Col>
            <Col sm="1">   </Col>
            <Col sm="2"> Team </Col>
            <Col sm="1"> DS   </Col>
            <Col sm="2"> ROBOT </Col>
            &nbsp;
            <Col> Battery </Col>
            <Col> RTT </Col>
          </Row>
          {
            this.props.stations?.map(s => 
              <AllianceStation
                key={s.station.station}
                station={s}
                state={this.props.state}
                onUpdate={(data) => this.props.onStationUpdate({ station: s.station, update: data })}
              />)
          }
        </Col>
      </Row>
    </div>
  }
}