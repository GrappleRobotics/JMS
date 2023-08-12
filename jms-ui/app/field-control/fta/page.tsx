"use client"
import { confirmModal } from "@/app/components/Confirm";
import "./fta.scss";
import { PermissionGate, withPermission } from "@/app/support/permissions"
import JmsWebsocket from "@/app/support/ws";
import { useWebsocket } from "@/app/support/ws-component";
import { AllianceStation, AllianceStationUpdate, ArenaState, DriverStationReport, Match, SerialisedLoadedMatch, Team } from "@/app/ws-schema";
import { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { faBattery, faCheck, faCode, faRobot, faTimes, faWifi } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import _ from "lodash";
import React, { useEffect, useState } from "react";
import { Button, Col, InputGroup, Row } from "react-bootstrap";
import { capitalise } from "@/app/support/strings";
import { useErrors } from "@/app/support/errors";
import update from "immutability-helper";
import BufferedFormControl from "@/app/components/BufferedFormControl";
import { MatchFlow } from "../match_flow";
import MatchScheduleControl from "../../match_schedule";

export default withPermission(["FTA", "FTAA"], function FTAView() {
  const [ allianceStations, setAllianceStations ] = useState<AllianceStation[]>([]);
  const [ dsReports, setDsReports ] = useState<{ [k: number]: DriverStationReport }>({});
  const [ state, setState ] = useState<ArenaState>({ state: "Init" });
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ teams, setTeams ] = useState<Team[]>([]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    let cb = [
      subscribe<"arena/stations">("arena/stations", setAllianceStations),
      subscribe<"arena/ds">("arena/ds", (reports) => setDsReports(_.keyBy(reports, "team"))),
      subscribe<"arena/state">("arena/state", setState),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
      subscribe<"matches/matches">("matches/matches", setMatches),
      subscribe<"team/teams">("team/teams", setTeams)
    ];
    return () => unsubscribe(cb);
  }, []);

  let remainingDsReports = { ...dsReports };
  let stationReports: (DriverStationReport | null)[] = [];

  for (let i = 0; i < allianceStations.length; i++) {
    let team = allianceStations[i].team;
    if (!team) {
      stationReports.push(null);
    } else {
      stationReports.push(remainingDsReports[team]);
      delete remainingDsReports[team];
    }
  }

  return <div style={{ marginLeft: '1em', marginRight: '1em' }}>
    <Row>
      {
        _.zip(allianceStations, stationReports).map(([stn, report]) => <FTAAllianceStation station={stn!} report={report || null} call={call} addError={addError} teams={teams} />)
      }
    </Row>
    {
      Object.keys(remainingDsReports).map(t => {
        const report = remainingDsReports[t as any];
        return <Row className="fta-remaining-ds-report">
          <Col>
            Team Connected but not in Match: { report.team } { report.actual_station && `(${capitalise(report.actual_station.alliance)} ${report.actual_station.station})` }
          </Col>
        </Row>
      })
    }
    <PermissionGate permissions={["FTA", "Scorekeeper"]}>
      <Row className="mt-3">
        <MatchFlow state={state} current_match={currentMatch} />
      </Row>
    </PermissionGate>
    <br />
    <MatchScheduleControl currentMatch={currentMatch || undefined} matches={matches} isLoadDisabled={state.state !== "Idle"} canLoad teams={teams} />
  </div>
});

function FTAAllianceStation({ station, report, call, addError, teams }: { station: AllianceStation, report: DriverStationReport | null, call: JmsWebsocket["call"], addError: (e: string) => void , teams?: Team[] }) {
  const diagnosis = ftaDiagnosis(station, report);

  const display_team = teams?.find(x => x.number === station.team)?.display_number;

  return <Col onClick={() => editStationModal(station, call, addError)} className="fta-alliance-station-col" data-alliance={station.id.alliance} data-bypass={station.bypass} data-estop={station.estop} data-astop={station.astop}>
    <Row>
      <Col md="auto"> <FTATeamIndicator ok={report?.rio_ping} icon={faRobot} /> </Col>
      <Col className="fta-alliance-station-team" data-has-administrative={station.team && display_team !== ("" + station.team)}>
        <Row>
          <Col> { display_team || station.team || "----" } </Col>
        </Row>
        {
          station.team && display_team !== ("" + station.team) && <Row>
            <Col> { station.team } </Col>
          </Row>
        }
      </Col>
      <Col md="auto"> <FTATeamIndicator ok={report?.robot_ping} icon={faCode} /> </Col>
    </Row>
    <Row>
      <Col>
        { diagnosis ? <span className="fta-diagnosis text-bad"> { diagnosis } </span> : <span className="fta-diagnosis text-good"> OK </span> }
      </Col>
    </Row>
    <Row className="fta-alliance-station-nstats">
      <Col>
        <FTATeamIndicator ok={(report?.battery_voltage || 0) > 9} icon={faBattery} text={`${report?.battery_voltage?.toFixed(2) || "--.--"}`} />
      </Col>
      <Col md="auto" className="p-0">
        <FTATeamIndicator ok={lostPktPercent(report?.pkts_lost, report?.pkts_sent) < 10} text={`${renderSent(report?.pkts_lost, report?.pkts_sent)}%`} />
      </Col>
      <Col>
        <FTATeamIndicator ok={report?.radio_ping} icon={faWifi} text={`${(report?.rtt?.toString() || "---").padStart(3, "\u00A0")}`} />
      </Col>
    </Row>
  </Col>
}

function ftaDiagnosis(station: AllianceStation, report: DriverStationReport | null) {
  if (station.bypass) return "BYP";
  if (station.astop) return "ASTOP";
  if (station.estop) return "ESTOP";
  if (report?.estop) return "R-EST";

  if (station.team === null) return "NOTEAM";

  if (report === null) return "NODS";
  if (report.actual_station !== null && station.id.alliance !== report.actual_station?.alliance && station.id.station != report.actual_station?.station) return "MOVE";

  if (!report.radio_ping) return "NORAD";
  if (!report.rio_ping) return "NORIO";
  if (!report.robot_ping) return "NOCODE";

  if (report.rtt > 100) return "L8NC";
  if (report.battery_voltage < 9) return "LBATT";

  return null;
}

function lostPktPercent(lost?: number, sent?: number) {
  return (lost || 0) / (((lost || 0) + (sent || 0)) || 1) * 100;
}

function renderSent(lost?: number, sent?: number) {
  let percent = 100 - lostPktPercent(lost, sent);

  if (percent > 100)
    return "HI";
  else if (percent < 0)
    return "LO";
  return percent.toFixed(0);
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

async function editStationModal(station: AllianceStation, call: JmsWebsocket["call"], addError: (e: string) => void) {
  let new_station = await confirmModal("", {
    title: `Edit ${capitalise(station.id.alliance)} ${station.id.station}`,
    data: station,
    renderInner: (data, onUpdate, ok, cancel) => <React.Fragment>
      <Button className="btn-block" size="lg" variant={data.bypass ? "danger" : "success"} onClick={() => onUpdate(update(data, { bypass: { $set: !data.bypass } }))}>
        { data.bypass ? "BYPASSED" : "Not Bypassed" }
      </Button>
      <hr />
      <Button className="btn-block" size="lg" variant="estop" onClick={() => { call<"arena/update_station">("arena/update_station", { station_id: station.id, updates: [ { estop: true } ] }); cancel() }}>
        EMERGENCY STOP { station.team || `${station.id.alliance.toUpperCase()} ${station.id.station}` }
      </Button>
      <hr />
      <InputGroup>
        <InputGroup.Text>Team Number</InputGroup.Text>
        <BufferedFormControl
          auto
          autofocus
          type="number"
          min={0}
          step={1}
          value={data.team || 0}
          onUpdate={v => onUpdate(update(data, { team: { $set: Math.floor(v as number) || null } }))}
        />
      </InputGroup>
    </React.Fragment>
  });

  let updates: AllianceStationUpdate[] = [];
  for (let key of Object.keys(new_station)) {
    if (key !== "id")
      updates.push({ [key]: (new_station as any)[key] } as any);
  }
  call<"arena/update_station">("arena/update_station", { station_id: station.id, updates })
    .catch(addError);
}