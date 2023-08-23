"use client";
import "./watch.scss";
import { withPermission } from "../support/permissions";
import React, { useEffect, useState } from "react";
import { AllianceStation, AllianceStationId, ArenaState, DriverStationReport, Match } from "../ws-schema";
import { useWebsocket } from "../support/ws-component";
import { useToasts } from "../support/errors";
import _ from "lodash";
import { ftaDiagnosis } from "../field-control/fta/page";
import { Button } from "react-bootstrap";
import moment from "moment";
import "moment-duration-format";

export default withPermission(["FTA"], function FTASmartWatch() {
  const [ allianceStations, setAllianceStations ] = useState<AllianceStation[]>([]);
  const [ dsReports, setDsReports ] = useState<{ [k: number]: DriverStationReport }>({});
  const [ arenaState, setArenaState ] = useState<ArenaState>({ state: "Init" });
  const [ nextMatch, setNextMatch ] = useState<Match | null>(null);
  const [ now, setNow ] = useState<moment.Moment>(moment());

  const [ actions, setActions ] = useState<boolean>(false);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

  useEffect(() => {
    let interval = setInterval(() => setNow(moment()), 500);
    let cb = [
      subscribe<"arena/stations">("arena/stations", setAllianceStations),
      subscribe<"arena/ds">("arena/ds", (reports) => setDsReports(_.keyBy(reports, "team"))),
      subscribe<"arena/state">("arena/state", setArenaState),
      subscribe<"matches/next">("matches/next", setNextMatch),
    ];
    return () => {
      clearInterval(interval);
      unsubscribe(cb);
    }
  }, []);

  const diagnoses: (string|null)[] = allianceStations.map(stn => {
    if (stn.team) {
      return ftaDiagnosis(stn, dsReports[stn.team] || null);
    } else {
      return "NOTEAM";
    }
  });

  const time_diff = nextMatch && moment(nextMatch.start_time).diff(now);
  let rendered_time = <React.Fragment />;
  // @ts-ignore
  let format = (d: moment.Duration) => d.format("d[d] h[h] m[m]", { trim: "both" });

  if (time_diff !== null && time_diff >= 0) {
    // @ts-ignore
    // rendered_time = `${moment.duration(time_diff).format("d[d] h[h] m[m]", { trim: "both" })} AHEAD`;
    rendered_time = <strong className="text-good">
      { format(moment.duration(time_diff)) } AHEAD
    </strong>
  } else if (time_diff !== null) {
    rendered_time = <strong className="text-bad">
      { format(moment.duration(-time_diff)) } BEHIND
    </strong>
  }

  return <div className="fta-watch-root" data-arena-state={arenaState.state} data-arena-ready={(arenaState as any)["ready"]}>
    {
      actions ? <div className="fta-watch">
        <div className="fta-watch-actions">
          {
            arenaState.state === "Estop" ? <Button variant="hazard-yellow" onClick={() => { call<"arena/signal">("arena/signal", { signal: "EstopReset" }).catch(addError); setActions(false) }}>
              RESET
            </Button> : <Button variant="estop" onClick={() => { call<"arena/signal">("arena/signal", { signal: "Estop" }).catch(addError); setActions(false) }}>
              ESTOP
            </Button>
          }
          <Button variant="secondary" onClick={() => setActions(false)}>
            Back
          </Button>
        </div>
      </div> : <div className="fta-watch" onClick={() => setActions(true)}>
        {
          arenaState.state === "Estop" ? <div className="fta-watch-headline"> ESTOP </div>
            : <React.Fragment>
                <div className="fta-watch-time">
                  { now.format("HH:MM:ss") }
                </div>
                <div className="fta-watch-team-issues">
                  {
                    _.zip(allianceStations, diagnoses).map(([stn, diag], i) => <div key={i} className="fta-watch-team-issue" data-ok={stn!.team === 9191} data-bypass={stn!.bypass} data-estop={stn!.estop} data-astop={stn!.astop} data-alliance={stn!.id.alliance} data-station={stn!.id.station}>
                      <div className="fta-watch-team"> { stn!.team || "----" } </div>
                      <div className="fta-watch-diagnosis"> { stn!.team === 9191 ? "OK" : (diag || "OK") } </div>
                    </div>)
                  }
                </div>
                <div className="fta-watch-match-timing">
                  { rendered_time }
                </div>
              </React.Fragment>
        }
      </div>
    }
  </div>
});