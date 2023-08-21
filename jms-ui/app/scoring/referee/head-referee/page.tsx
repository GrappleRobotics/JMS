"use client"
import { useToasts } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { Match, MatchScoreSnapshot, SerialisedLoadedMatch } from "@/app/ws-schema";
import { useEffect, useState } from "react";
import { Button, Col, Row } from "react-bootstrap";
import { RefereePanelFouls } from "../referee";
import { ALLIANCES } from "@/app/support/alliances";

export default withPermission(["Scoring"], function HeadReferee() {
  const [ score, setScore ] = useState<MatchScoreSnapshot>();
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null)

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();
  
  useEffect(() => {
    let cbs = [
      subscribe<"scoring/current">("scoring/current", setScore),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
      subscribe<"matches/matches">("matches/matches", setMatches)
    ];
    return () => unsubscribe(cbs);
  }, []);
  
  return <div className="referee-panel">
    <Row className="mb-3 mt-3">
      <Col>
        <h3 className="mb-0"><i>{ currentMatch ? currentMatch.match_id === "test" ? "Test Match" : matches.find(m => m.id === currentMatch?.match_id)?.name || currentMatch?.match_id : "Waiting for Scorekeeper..." }</i></h3>
        <i className="text-muted"> HEAD REFEREE </i>
      </Col>
    </Row>
    { score && <RefereePanelFouls
      score={score}
      onUpdate={update => call<"scoring/score_update">("scoring/score_update", { update: update }).then(setScore).catch(addError)}
      flipped={false}
    /> }
    <Row>
      {
        score && ALLIANCES.map(alliance => <Col key={alliance as string}>
          <Button
            className="btn-block referee-station-score"
            data-score-type="charge_station_level"
            data-score-value={score[alliance].live.charge_station_level.auto}
            onClick={() => call<"scoring/score_update">("scoring/score_update", { update: { alliance, update: { ChargeStationLevel: { auto: true, level: !score[alliance].live.charge_station_level.auto } } } })}
          >
            { score[alliance].live.charge_station_level.auto ? "CS LEVEL AUTO" : "CS NOT LEVEL AUTO" }
          </Button>

          <Button
            className="mt-2 btn-block referee-station-score"
            data-score-type="charge_station_level"
            data-score-value={score[alliance].live.charge_station_level.teleop}
            onClick={() => call<"scoring/score_update">("scoring/score_update", { update: { alliance, update: { ChargeStationLevel: { auto: false, level: !score[alliance].live.charge_station_level.teleop } } } })}
          >
            { score[alliance].live.charge_station_level.teleop ? "CS LEVEL TELEOP" : "CS NOT LEVEL TELEOP" }
          </Button>
        </Col>)
      }
    </Row>
  </div>
})