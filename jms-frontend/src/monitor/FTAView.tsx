import { IconDefinition, faCode, faNetworkWired, faRobot, faWifi } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { confirmModal } from "components/elements/Confirm";
import MatchFlow from "match_control/MatchFlow";
import React from "react";
import { Button, Col, Modal, Row } from "react-bootstrap";
import { capitalise } from "support/strings";
import { ALLIANCES, ALLIANCE_STATIONS } from "support/ws-additional";
import { WebsocketComponent } from "support/ws-component";
import { Alliance, ArenaState, LoadedMatch, ResourceRequirementStatus, SerialisedAllianceStation } from "ws-schema";

type FTAViewState = {
  stations: SerialisedAllianceStation[],
  state?: ArenaState,
  match?: LoadedMatch,
  resource_status?: ResourceRequirementStatus,
}

export default class FTAView extends WebsocketComponent<{ fta: boolean }, FTAViewState> {
  readonly state: FTAViewState = { stations: [] };

  componentDidMount = () => this.handles = [
    this.listen("Arena/State/Current", "state"),
    this.listen("Arena/Match/Current", "match"),
    this.listen("Arena/Alliance/CurrentStations", "stations"),
    this.listen("Resource/Requirements/Current", "resource_status")
  ];

  render() {
    const hasMatch = !!this.state.match;
    return <div className="fta-view">
      <Row>
        {
          this.state.stations.map(station => <Col className="fta-alliance-station-col">
            <FTAAllianceStation station={station} state={this.state.state} match={this.state.match} />
          </Col>)
        }
      </Row>
      <Row>
        <Col className="fta-match-flow">
          <MatchFlow
            state={this.state.state}
            matchLoaded={hasMatch}
            onSignal={sig => this.send({ Arena: { State: { Signal: sig } } })}
            onAudienceDisplay={scene => this.send({ Arena: { AudienceDisplay: { Set: scene } } })}
            resources={this.state.resource_status}
            stations={this.state.stations}
          />
        </Col>
      </Row>
    </div>
  }
}

type FTAAllianceStationProps = {
  station: SerialisedAllianceStation,
  state?: ArenaState,
  match?: LoadedMatch
}

class FTAAllianceStation extends React.PureComponent<FTAAllianceStationProps> {
  onPopup = async () => {
    const { station } = this.props;
    await confirmModal("", {
      cancelIfBackdrop: true,
      size: "xl",
      title: `${capitalise(station.station.alliance)} ${station.station.station} - ${station.team || "Unoccupied"}`,
      render: (ok: (data?: any) => void, cancel: () => void) => <React.Fragment>
        <Modal.Body className="fta-team-modal">
          Hello!
        </Modal.Body>
        <Modal.Footer>
          <Button size="lg" className="btn-block" variant="secondary" onClick={cancel}>Cancel</Button>
        </Modal.Footer>
      </React.Fragment>
    })
  }

  lostPktPercent = (lost?: number, sent?: number) => (lost || 0) / (((lost || 0) + (sent || 0)) || 1) * 100;

  renderSent = (lost?: number, sent?: number) => {
    let percent = 100 - this.lostPktPercent(lost, sent);

    if (percent > 100)
      return "HI";
    else if (percent < 0)
      return "LO";
    return percent.toFixed(0);
  }

  diagnosis = () => {
    let stn = this.props.station;
    let report = this.props.station.ds_report;
    let playing_match = this.props.state?.state == "MatchPlay";

    if (stn.bypass) return "BYP";
    if (stn.estop) return "ESTOP";
    if (stn.astop) return "ASTOP";
    if (report?.estop) return "R-EST";

    if (stn.team == null) return "NOTEAM";
    if (stn.occupancy == "Vacant") return "NODS";
    if (stn.occupancy == "WrongMatch") return "WRMAT";
    if (stn.occupancy == "WrongStation") return "MOVE";

    if (report == null) return "NOREP";

    if (!report.radio_ping) return "NORAD";
    if (!report.rio_ping) return "NORIO";
    if (!report.robot_ping) return "NOCODE";

    if (report.rtt > 100) return "LATNC";
    if (report.battery < 9) return "LBATT";

    if (playing_match && this.lostPktPercent(report.pkts_lost, report.pkts_sent) > 10)
      return "PKTLOS";

    if (playing_match && report.mode == null) return "DSLBD";
    if (playing_match && report.mode != this.props.match?.state) return "BADMD";
    
    return null;
  }

  render() {
    const { station } = this.props;

    const diagnosis = this.diagnosis();

    return (
      <div
        className="fta-alliance-station"
        data-alliance={station.station.alliance}
        data-bypass={station.bypass}
        data-estop={station.estop}
        data-astop={station.astop}
        onClick={() => this.onPopup()}
      >
        <Row>
          <Col md="auto">
            <FTATeamIndicator ok={station.ds_report?.rio_ping} icon={faRobot} />
          </Col>
          <Col className="col-full fta-alliance-station-team">
            { station.team || "----" }
          </Col>
          <Col md="auto">
            <FTATeamIndicator ok={station.ds_report?.robot_ping} icon={faCode} />
          </Col>
        </Row>
        <Row>
          <Col md="auto">
            <FTATeamIndicator ok={station.ds_eth} icon={faNetworkWired} />
          </Col>
          <Col>
            {
              diagnosis ? <span className="fta-diagnosis text-bad"> { diagnosis } </span> : <span className="fta-diagnosis text-good"> OK </span>
            }
          </Col>
          <Col md="auto">
            <FTATeamIndicator ok={station.ds_report?.radio_ping} icon={faWifi} />
          </Col>
        </Row>
        <Row className="fta-alliance-station-nstats">
          <Col>
            <FTATeamIndicator ok={(station.ds_report?.battery || 0) > 9} text={`${station.ds_report?.battery?.toFixed(2) || "--.--"} V`} />
          </Col>
          <Col md="auto">
            <FTATeamIndicator ok={this.lostPktPercent(station.ds_report?.pkts_lost, station.ds_report?.pkts_sent) < 10} text={`${this.renderSent(station.ds_report?.pkts_lost, station.ds_report?.pkts_sent)}%`} />
          </Col>
          <Col>
            <FTATeamIndicator ok={station.ds_report?.radio_ping} text={`${(station.ds_report?.rtt?.toString() || "---").padStart(3, "\u00A0")} ms`} />
          </Col>
        </Row>
      </div>
    )
  }
}

type FTATeamIndicatorProps = {
  ok?: boolean,
  icon?: IconDefinition,
  text?: string
};

class FTATeamIndicator extends React.PureComponent<FTATeamIndicatorProps> {
  render() {
    return <div className="fta-team-indicator" data-ok={this.props.ok}>
      { this.props.icon && <span className="icon"><FontAwesomeIcon icon={this.props.icon} /></span> }
      { this.props.text && <React.Fragment>
        &nbsp; { this.props.text }
      </React.Fragment> }
    </div>
  }
}