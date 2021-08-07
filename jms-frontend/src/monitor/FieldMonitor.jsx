import { faCarBattery, faCode, faRobot, faSkullCrossbones, faWifi } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Col, Row } from "react-bootstrap";

class FieldMonitorStation extends React.PureComponent {
  lostPktPercent = (lost, sent) => (lost || 0) / ((lost + sent) || 1) * 100;

  renderSent = (lost, sent) => {
    let percent = 100 - this.lostPktPercent(lost, sent);

    if (percent > 100)
      return "HI";
    else if (percent < 0)
      return "LO";
    return percent.toFixed(0);
  }

  whatError = (stn, report, estop) => {
    if (stn.bypass) return null;
    if (estop) return "E-STOPPED";

    if (stn.team === null) return "NO TEAM";
    if (stn.occupancy == "Vacant") return "NO DS";
    if (stn.occupancy == "WrongMatch") return "WRONG MATCH";
    if (stn.occupancy == "WrongStation") return "WRONG STATION";

    if (!report) return "NO DS REPORT";

    if (!report.radio_ping) return "NO RADIO";
    if (!report.rio_ping) return "NO RIO";
    if (!report.robot_ping) return "NO CODE";

    if (report.rtt > 100) return "HIGH LATENCY";

    if (report.battery < 9) return "LOW BATTERY";

    if (report.mode !== null && this.lostPktPercent(report.pkts_lost, report.pkts_sent) > 10)
      return "HIGH PACKET LOSS";

    return null;
  }

  render() {
    let s = this.props.station;
    let report = s.ds_report;
    let estop = s.estop || s.astop || report?.estop;

    let error = this.whatError(s, report, estop);

    return <Row className="monitor-station" data-error={error} data-bypass={s.bypass} data-estop={estop}>
      <Col className="monitor-team" md="auto">
        { s.team || <i className="text-muted"> --- </i> }
      </Col>
      <Col className="monitor-data col-full">
        {
          s.bypass ? <React.Fragment /> : 
            <Row className="monitor-data-header">
              <Col className="monitor-occupancy" data-occupancy={s.occupancy} />
              <Col className="text-center" data-ok={s.ds_report ? (s.ds_report.mode ? "true" : "false") : ""}>
                { s.ds_report?.estop ? "ESTOP" : (s.ds_report?.mode?.toUpperCase() || "DISABLED") }
              </Col>
              <Col className="text-right" data-ok={ this.lostPktPercent(report?.pkts_lost, report?.pkts_sent) < 10 }>
                { this.renderSent(s.ds_report?.pkts_lost, s.ds_report?.pkts_sent) }%
              </Col>
            </Row>
        }
        <Row className="monitor-error align-items-center">
          <Col>
            { s.bypass ? "BYPASS" : (error || "") }
          </Col>
        </Row>
        {
          s.bypass ? <React.Fragment /> : 
            <Row className="monitor-indicators">
              <Col data-ok={s.ds_report?.radio_ping} md="4">
                <FontAwesomeIcon icon={faWifi} /> &nbsp;
                { (s.ds_report?.rtt?.toString() || "---").padStart(3, "\u00A0") }ms
              </Col>
              <Col data-ok={s.ds_report?.rio_ping}>
                <FontAwesomeIcon icon={faRobot} size="lg"/> &nbsp;
              </Col>
              <Col data-ok={s.ds_report?.robot_ping}>
                <FontAwesomeIcon icon={faCode} size="lg"/> &nbsp;
              </Col>
              <Col data-ok={s.ds_report?.battery ? s.ds_report.battery > 9 : ""} md="auto">
                <FontAwesomeIcon icon={faCarBattery} size="lg"/> &nbsp;
                { s.ds_report?.battery?.toFixed(2) || "--.--" } V
              </Col>
            </Row>
        }
      </Col>
    </Row>
  }
}

export default class FieldMonitor extends React.PureComponent {
  constructor(props) {
    super(props);

    props.ws.subscribe("arena", "stations");
  }

  renderAlliance = (stations) => {
    return <React.Fragment>
      {
        stations.map(s => <FieldMonitorStation station={s}/>)
      }
    </React.Fragment>
  }

  renderAvailable = () => {
    return <Row>
      <Col className="col-full monitor-alliance" data-alliance="blue">
        { this.renderAlliance( this.props.arena.stations.filter(s => s.station.alliance == "Blue") ) }
      </Col>
      <Col className="col-full monitor-alliance" data-alliance="red">
        { this.renderAlliance( this.props.arena.stations.filter(s => s.station.alliance == "Red").reverse() ) }
      </Col>
    </Row>
  }

  render() {
    return <Col className="col-full">
      {
        this.props.arena?.stations ? this.renderAvailable() : <h4 className="m-5"> Waiting... </h4>
      }
    </Col>
  }
}