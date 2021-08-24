import React from 'react';
import { Row, Col, Button } from 'react-bootstrap';
import BufferedFormControl from 'components/elements/BufferedFormControl';
 
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

  renderModeIndicator = (report) => {
    if (report.estop) {
      return <Indicator variant="danger"> E </Indicator>;
    } else {
      switch (report.mode) {
        case "Auto":
          return <Indicator variant="warning"> A </Indicator>;
        case "Teleop":
          return <Indicator variant="info"> T </Indicator>;
        case "Test":
          return <Indicator variant="success"> ~ </Indicator>;
        default:
          return <Indicator variant="dark"> </Indicator>;
      }
    }
  }
  
  render() {
    let match_loaded = this.props.match_loaded;
    let s = this.props.station;
    let cname = ["alliance-station"];
    let can_bypass = match_loaded && (this.props.state?.state === "Prestart" || this.props.state?.state === "Idle");
    let can_change_team = match_loaded && this.props.state?.state === "Idle";
    let is_estoppable = match_loaded && (this.props.state?.state === "MatchPlay");

    if (s.astop)
      cname.push("bg-hazard-dark-active");
    if (s.estop)
      cname.push("bg-hazard-red-dark-active");
    if (s.bypass)
      cname.push("bypassed");

    let report = s.ds_report;

    return <Row className={cname.join(" ")} {...this.props}>
      <Col sm="1">
        <i>{ s.station.station }</i>
      </Col>
      <Col sm="1" className="p-0">
        {
          is_estoppable ? <Button
            size="sm"
            variant="hazard-red-dark"
            disabled={ s.estop }
            onClick={() => this.props.onUpdate({ estop: true })}
          >
            E
          </Button> 
          : <Button
            size="sm"
            variant={s.bypass ? "success" : (can_bypass ? "danger" : "dark")}
            disabled={ !can_bypass }
            onClick={() => this.props.onUpdate({ bypass: !s.bypass })}
          >
              BY
          </Button>
        }
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
          report ?
            <Indicator variant={s.ds_report.robot_ping ? "success" : "danger"} /> :
            <Indicator variant="dark" />
        }

        {
          report ? 
            <Indicator variant={s.ds_report.rio_ping ? "success" : "danger"} /> : 
            <Indicator variant="dark" />
        }
      </Col>
      <Col sm={3}>
        <small>
          {
            (report?.battery?.toFixed(1) || "--.-").padStart(4, "\u00A0") + " V"
          }
          <span className="text-muted">
            &nbsp;/&nbsp;
          </span>
          {
            (report?.rtt?.toString() || "-").padStart(3, "\u00A0")
          }
        </small>
      </Col>
      <Col>
        {
          report ? this.renderModeIndicator(report) : <Indicator variant="dark" />
        }
      </Col>
    </Row>
  }
}

export default class Alliance extends React.Component {
  renderScore = (score) => {
    let score_components = [
      <Col>
        <h6> Stage { score.derived.stage } </h6>
      </Col>,
      <Col>
        <h6> {score.derived.total_bonus_rp} rp </h6>
      </Col>,
      <Col>
        <h4> { score.derived.total_score } </h4>
      </Col>
    ]

    return <React.Fragment>
      { this.props.flipped ? score_components.reverse() : score_components }
    </React.Fragment>
  }

  render() {
    return <div className={"p-3 alliance-" + this.props.colour.toLowerCase()}>
      <Row>
        <Col>
          <Row className={"alliance-score px-4 " + (this.props.flipped ? "text-left" : "text-right")}>
            {
              this.props.score ? this.renderScore(this.props.score) : <React.Fragment />
            }
          </Row>
          <Row className="alliance-header">
            <Col sm="1"> # </Col>
            <Col sm="1">   </Col>
            <Col sm="2"> Team </Col>
            <Col sm="1"> DS   </Col>
            <Col sm="2"> ROBOT </Col>
            <Col sm="3"> Batt / RTT </Col>
            <Col> Mode </Col>
          </Row>
          {
            this.props.stations?.map(s => 
              <AllianceStation
                key={s.station.station}
                match_loaded={this.props.match_loaded}
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