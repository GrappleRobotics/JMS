"use client"
import { confirmModal } from "@/app/components/Confirm";
import "./fta.scss";
import { PermissionGate, withPermission } from "@/app/support/permissions"
import JmsWebsocket from "@/app/support/ws";
import { useWebsocket } from "@/app/support/ws-component";
import { AllianceStation, AllianceStationUpdate, ArenaState, DriverStationReport, Match, SerialisedLoadedMatch, SupportTicket, Team } from "@/app/ws-schema";
import { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import { faBattery, faCheck, faCode, faFlag, faNetworkWired, faRobot, faSign, faTimes, faWifi } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import _ from "lodash";
import React, { useEffect, useState } from "react";
import { Button, Card, Col, InputGroup, ListGroup, Row } from "react-bootstrap";
import { capitalise } from "@/app/support/strings";
import { useErrors } from "@/app/support/errors";
import update from "immutability-helper";
import BufferedFormControl from "@/app/components/BufferedFormControl";
import { MatchFlow } from "../match_flow";
import MatchScheduleControl from "../../match_schedule";
import { newTicketModal } from "@/app/csa/tickets";
import FloatingActionButton from "@/app/components/FloatingActionButton";
import Paginate from "@/app/components/Paginate";
import Link from "next/link";
import moment from "moment";

export default withPermission(["FTA", "FTAA"], function FTAView() {
  const [ allianceStations, setAllianceStations ] = useState<AllianceStation[]>([]);
  const [ dsReports, setDsReports ] = useState<{ [k: number]: DriverStationReport }>({});
  const [ state, setState ] = useState<ArenaState>({ state: "Init" });
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ teams, setTeams ] = useState<Team[]>([]);
  const [ signboard, setSignboard ] = useState<string | null>(null);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    let cb = [
      subscribe<"arena/stations">("arena/stations", setAllianceStations),
      subscribe<"arena/ds">("arena/ds", (reports) => setDsReports(_.keyBy(reports, "team"))),
      subscribe<"arena/state">("arena/state", setState),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
      subscribe<"matches/matches">("matches/matches", setMatches),
      subscribe<"team/teams">("team/teams", setTeams),
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
        _.zip(allianceStations, stationReports).map(([stn, report], i) => <FTAAllianceStation key={i} station={stn!} match_id={currentMatch?.match_id} report={report || null} call={call} addError={addError} teams={teams} />)
      }
    </Row>
    {
      Object.keys(remainingDsReports).map((t, i) => {
        const report = remainingDsReports[t as any];
        return <Row className="fta-remaining-ds-report" key={i}>
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

    <FloatingActionButton icon={faSign} variant="purple" onClick={() => signboardModal().then(signboard => setSignboard(signboard))} />
    {
      signboard && <div className="fta-signboard" onClick={() => setSignboard(null)}>
        { signboard }
      </div>
    }
  </div>
});

function FTAAllianceStation({ station, report, match_id, call, addError, teams }: { station: AllianceStation, report: DriverStationReport | null, match_id?: string, call: JmsWebsocket["call"], addError: (e: string) => void , teams?: Team[] }) {
  const [ tickets, setTickets ] = useState<SupportTicket[]>([]);
  const diagnosis = ftaDiagnosis(station, report);

  const display_team = teams?.find(x => x.number === station.team)?.display_number;

  useEffect(() => {
    if (station.team) {
      call<"tickets/all">("tickets/all", null)
        .then(ticks => setTickets(ticks.filter(t => t.team === station.team)))
        .catch(addError)
    } else {
      setTickets([]);
    }
  }, [ station.team ]);

  return <Col onClick={() => editStationModal(station, match_id, tickets, call, addError)} className="fta-alliance-station-col" data-alliance={station.id.alliance} data-bypass={station.bypass} data-estop={station.estop} data-astop={station.astop}>
    <Row className="mx-0">
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
    <Row className="mx-0">
      <Col md="auto">
        <FTATeamIndicator ok={station.ds_eth_ok === null ? undefined : station.ds_eth_ok} icon={faNetworkWired} />
      </Col>
      <Col>
        { diagnosis ? <span className="fta-diagnosis text-bad"> { diagnosis } </span> : <span className="fta-diagnosis text-good"> OK </span> }
      </Col>
      <Col md="auto">
        <FTATeamIndicator ok={tickets.length === 0} icon={faFlag} okVariant="dark" badVariant="yellow" />
      </Col>
    </Row>
    <Row className="fta-alliance-station-nstats mx-0">
      <Col>
        <FTATeamIndicator ok={report?.battery_voltage ? report.battery_voltage > 9 : undefined} icon={faBattery} text={`${report?.battery_voltage?.toFixed(2) || "--.--"}`} />
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
  text?: string,
  okVariant?: string,
  badVariant?: string,
  unknownVariant?: string,
};

class FTATeamIndicator extends React.PureComponent<FTATeamIndicatorProps> {
  render() {
    return <div className={`fta-team-indicator text-${this.props.ok === undefined ? (this.props.unknownVariant || "dark") : this.props.ok ? (this.props.okVariant || "good") : (this.props.badVariant || "bad")}`}>
      { this.props.icon && <span className="icon"><FontAwesomeIcon icon={this.props.icon} /></span> }
      { this.props.text && <React.Fragment>
        &nbsp; { this.props.text }
      </React.Fragment> }
    </div>
  }
}

async function editStationModal(station: AllianceStation, match_id: string | undefined, tickets: SupportTicket[], call: JmsWebsocket["call"], addError: (e: string) => void) {
  let new_station = await confirmModal("", {
    title: `Edit ${capitalise(station.id.alliance)} ${station.id.station}`,
    data: station,
    size: "xl",
    renderInner: (data, onUpdate, ok, cancel) => <Row>
      <Col md={4}>
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
        <hr />
        <Button className="btn-block" size="lg" variant={data.bypass ? "danger" : "success"} onClick={() => onUpdate(update(data, { bypass: { $set: !data.bypass } }))}>
          { data.bypass ? "BYPASSED" : "Not Bypassed" }
        </Button>
        <hr />
        <Button className="btn-block" size="lg" variant="estop" onClick={() => { call<"arena/update_station">("arena/update_station", { station_id: station.id, updates: [ { estop: true } ] }); cancel() }}>
          EMERGENCY STOP { station.team || `${station.id.alliance.toUpperCase()} ${station.id.station}` }
        </Button>
        <hr />
        { station.team && match_id && <Button className="btn-block" size="lg" variant="orange" onClick={() => { newTicketModal(station.team!, match_id, call, addError); cancel() }}>
          Flag Issue for CSA
        </Button> }
      </Col>
      <Col>
        <h5> Previous Issues </h5>
        { tickets.length > 0 ? <Paginate itemsPerPage={5}>
          {
            tickets.map(ticket => <Link className="no-text-decoration" key={ticket.id} href={`/csa/${ticket.id}`}>
              <Row className="fta-ticket">
                <Col md={3}>
                  <strong>{ ticket.resolved && <FontAwesomeIcon className="text-success" icon={faCheck} /> }&nbsp; { ticket.issue_type } { ticket.match_id ? `in ${ticket.match_id}` : "" } </strong>
                </Col>
                <Col>
                  { ticket.notes.map((note, i) => <Row 
                    key={i}
                    className="fta-ticket-note text-white mb-0"
                  >
                    <Col md={2} className="text-muted"> { moment(note.time).fromNow() } </Col>
                    <Col> { note.comment } </Col>
                  </Row>) }
                </Col>
              </Row>
            </Link>)
          }
        </Paginate> : <p className="text-muted">The record is pretty clean...</p> }
      </Col>
    </Row>
  });

  let updates: AllianceStationUpdate[] = [];
  for (let key of Object.keys(new_station)) {
    if (key !== "id")
      updates.push({ [key]: (new_station as any)[key] } as any);
  }
  call<"arena/update_station">("arena/update_station", { station_id: station.id, updates })
    .catch(addError);
}

const SIGNBOARD_PRESETS = [
  "Turn Your Robot On",
  "Plug In Your Laptop",
  "Turn Your Laptop On",
  "Please Come Here",
  "Plug In Your Radio",
  "Plug In Your RoboRIO",
  "Come Check Your Wires",
]

async function signboardModal() {
  return confirmModal("", {
    data: "",
    size: "lg",
    title: "Select Signboard Text",
    okText: "Display",
    renderInner: (text, setText, ok) => <React.Fragment>
      {
        SIGNBOARD_PRESETS.map((sbp, i) => <Button key={i} className="btn-block mb-2" size="lg" onClick={() => ok(sbp)}>
          { sbp }
        </Button>)
      }
      <InputGroup>
        <InputGroup.Text>Custom</InputGroup.Text>
        <BufferedFormControl
          instant
          value={text}
          onUpdate={v => setText(v as string)}
          placeholder={"Please Turn your Robot On"}
          size="lg"
        />
      </InputGroup>
    </React.Fragment>
  })
}