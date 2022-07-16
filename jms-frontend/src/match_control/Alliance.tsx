import { IconProp } from '@fortawesome/fontawesome-svg-core';
import { faBan, faBugSlash, faCode, faGear, faShuffle, faXmark } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import BufferedFormControl from 'components/elements/BufferedFormControl';
import SimpleTooltip from 'components/elements/SimpleTooltip';
import React from 'react';
import { Button, Col, Row } from 'react-bootstrap';
import { AllianceStation, AllianceStationDSReport, AllianceStationOccupancy, ArenaMessageAlliance2JMS, ArenaState, DSMode, SnapshotScore } from 'ws-schema';

type IndicatorProps = React.HTMLAttributes<HTMLDivElement>;

class Indicator extends React.PureComponent<IndicatorProps> {
  render() {
    let {className, children, ...props} = this.props; 
    return <div className={`indicator ${className || ""}`} {...props}> { children } </div>
  }
}

const DSStatusIndicator = ( { className, occupancy, ...props }: IndicatorProps & { occupancy: AllianceStationOccupancy } ) => {
  const status_map = {
    "Vacant": { icon: undefined, tip: "Driver Station Vacant" },
    "Occupied": { icon: undefined, tip: "Driver Station OK" },
    "WrongStation": { icon: faShuffle, tip: "Wrong Station" },
    "WrongMatch": { icon: faXmark, tip: "Wrong Match" }
  };

  const status = status_map[occupancy];

  return <SimpleTooltip id="ds-status-tt" tip={ status.tip }>
    <Indicator className={`ds-status ${className || ""}`} data-occupancy={occupancy} {...props}>
      { status.icon ? <FontAwesomeIcon icon={status.icon} />  : <React.Fragment /> }
    </Indicator>
  </SimpleTooltip>
}

const DSModeIndicator = ( { className, report, ...props }: IndicatorProps & { report?: AllianceStationDSReport } ) => {
  type ModeT = DSMode | "Estop" | "Unknown"
  let mode: ModeT = report?.estop ? "Estop" : report?.mode || "Unknown";

  const mode_map: { [k in ModeT]: { tip: string, icon?: IconProp, content?: string } } = {
    "Unknown": { tip: "No DS Report" },
    "Estop": { icon: faBan, tip: "DS - Emergency Stop" },
    "Teleop": { content: "T", tip: "DS - Teleop" },
    "Auto": { content: "A", tip: "DS - Auto" },
    "Test": { icon: faBugSlash, tip: "DS - Test" }
  };

  const { icon, content, tip } = mode_map[mode];

  return <SimpleTooltip id="ds-mode-tt" tip={tip}>
    <Indicator className={`ds-mode ${className || ""}`} data-mode={mode} {...props}>
      {
        icon ? <FontAwesomeIcon icon={icon} size="sm" /> : (content || "")
      }
    </Indicator>
  </SimpleTooltip>
}

type AllianceStationProps = {
  matchLoaded: boolean,
  arenaState: ArenaState,
  station: AllianceStation,
  onUpdate: (update: ArenaMessageAlliance2JMS["UpdateAlliance"]) => void
}

class AllianceStationComponent extends React.PureComponent<AllianceStationProps> {
  doUpdate = (update: Omit<ArenaMessageAlliance2JMS["UpdateAlliance"], "station">) => {
    this.props.onUpdate({ station: this.props.station.station, ...update })
  }
  
  render() {
    let {matchLoaded, arenaState, station, onUpdate, ...props} = this.props;
    let can_bypass = matchLoaded && (arenaState.state === "Prestart" || arenaState.state === "Idle");
    let can_change_team = matchLoaded && (arenaState.state === "Idle");
    let is_estoppable = matchLoaded && (arenaState.state === "MatchPlay");

    let report = station.ds_report;

    return <Row 
      className="alliance-station"
      data-astop={station.astop}
      data-estop={station.estop}
      data-bypass={station.bypass}
      {...props}
    >
      <Col sm="1">
        <i>{ station.station.station }</i>
      </Col>
      <Col sm="1" className="p-0">
        {
          is_estoppable ? <Button
            size="sm"
            variant="estop"
            disabled={ station.estop }
            onClick={() => this.doUpdate({ estop: true })}
          >
            E
          </Button> 
          : <Button
            size="sm"
            variant={station.bypass ? "success" : (can_bypass ? "danger" : "dark")}
            disabled={ !can_bypass }
            onClick={() => this.doUpdate({ bypass: !station.bypass })}
          >
              BY
          </Button>
        }
      </Col>
      <Col sm="2">
        <BufferedFormControl
          className="team-num"
          type="number"
          onUpdate={(n) => this.doUpdate({ team: parseInt(""+n) })}
          value={station.team || ""}
          disabled={ !can_change_team }
          placeholder={"----"}
        />
      </Col>
      <Col sm="1">
        <DSStatusIndicator occupancy={station.occupancy} />
      </Col>
      <Col sm="2">
        <SimpleTooltip id="robot-ping-tt" tip={report?.robot_ping ? "Robot Comms OK" : "No Robot Comms"}>
          <Indicator className="ping" data-bool-value={ report?.robot_ping }>
            <FontAwesomeIcon icon={faGear} size="xs" />
          </Indicator>
        </SimpleTooltip>

        <SimpleTooltip id="rio-ping-tt" tip={report?.rio_ping ? "Code Running" : "Code Crashed / Not Running"}>
          <Indicator className="ping" data-bool-value={ report?.rio_ping }>
            <FontAwesomeIcon icon={faCode} size="xs" />
          </Indicator>
        </SimpleTooltip>
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
        <DSModeIndicator report={report || undefined} />
      </Col>
    </Row>
  }
}

type AllianceProps = {
  colour: "red" | "blue",
  matchLoaded: boolean,
  arenaState?: ArenaState,
  matchScore?: SnapshotScore,
  stations: AllianceStation[],
  onStationUpdate: (update: ArenaMessageAlliance2JMS["UpdateAlliance"]) => void
}

export default class AllianceDash extends React.PureComponent<AllianceProps> {
  renderScore = (score: SnapshotScore) => {
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
      { this.props.colour === "red" ? score_components.reverse() : score_components }
    </React.Fragment>
  }

  render() {
    let { colour, matchLoaded, arenaState, matchScore, stations } = this.props;
    return <div className="alliance" data-colour={colour}>
      <Row>
        <Col>
          <Row className={"alliance-score px-4 " + (colour === "red" ? "text-left" : "text-end")}>
            {
              matchScore ? this.renderScore(matchScore) : <React.Fragment />
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
            stations?.map(s => 
              arenaState ? 
                <AllianceStationComponent
                  key={s.station.station}
                  matchLoaded={matchLoaded}
                  station={s}
                  arenaState={arenaState}
                  onUpdate={this.props.onStationUpdate} />
                : <React.Fragment />)
          }
        </Col>
      </Row>
    </div>
  }
}