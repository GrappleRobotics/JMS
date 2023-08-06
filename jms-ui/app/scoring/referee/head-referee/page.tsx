"use client"
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { Match, MatchScoreSnapshot, SerialisedLoadedMatch } from "@/app/ws-schema";
import { useEffect, useState } from "react";
import { Col, Row } from "react-bootstrap";
import { RefereePanelFouls } from "../referee";

export default withPermission(["Scoring"], function HeadReferee() {
  const [ score, setScore ] = useState<MatchScoreSnapshot>();
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null)

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();
  
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
  </div>
})