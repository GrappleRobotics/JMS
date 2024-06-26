"use client"

import { useToasts } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { Alliance, Match, SerialisedLoadedMatch, SnapshotScore } from "@/app/ws-schema";
import { useEffect, useState } from "react";
import { Button, Col, Row } from "react-bootstrap";

export default withPermission(["HumanPlayerRed", "HumanPlayerBlue"], function HumanPlayerPanel({ params }: { params: { alliance: Alliance } }) {
  const [ score, setScore ] = useState<SnapshotScore>();
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null)

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

  useEffect(() => {
    let cbs = [
      subscribe<"scoring/current">("scoring/current", s => setScore(s[params.alliance])),
      subscribe<"arena/current_match">("arena/current_match", (s) => {
        setCurrentMatch(s);
      }),
      subscribe<"matches/matches">("matches/matches", setMatches)
    ];
    return () => unsubscribe(cbs);
  }, []);

  const coop_enabled = !score?.live?.coop && (score?.live?.notes?.banked || 0) > 0 && (currentMatch?.match_time || 0) < 45 * 1000;
  const amplify_enabled = (score?.live?.notes?.banked || 0) >= 2;

  return <div className="hp-panel">
    <Row className="mb-3 mt-3">
      <Col>
        <h3 className="mb-0"><i>{ currentMatch ? currentMatch.match_id === "test" ? "Test Match" : matches.find(m => m.id === currentMatch?.match_id)?.name || currentMatch?.match_id : "Waiting for Scorekeeper..." }</i></h3>
        <i className="text-muted"> { params.alliance.toUpperCase() } Human Player </i>
      </Col>
    </Row>

    {
      currentMatch && <Row>
        <Col>
          <h3> Co-op: { coop_enabled ? "READY" : "NOT READY" } </h3>
          <Button
            className="btn-block hp-button"
            data-button-type="2024-coop"
            data-alliance={params.alliance}
            variant="yellow"
            disabled={!coop_enabled}
            onClick={() => {
              if (coop_enabled)
                call<"scoring/score_update">("scoring/score_update", { update: { alliance: params.alliance, update: "Coop" } })
            }}
          >
            CO-OP
          </Button>
          <br />
          <h5> { score?.live?.coop ? <span className="text-good">CO-OP PRESSED</span> : <span className="text-muted">Co-op Not Pressed</span> } </h5>
          <h5> <span className="text-muted">(other alliance unknown)</span> </h5>
        </Col>
        <Col>
          <h3> Banked: { score?.live?.notes?.banked || 0 }</h3>
          <Button
            className="btn-block hp-button"
            data-button-type="2024-amplify"
            data-alliance={params.alliance}
            variant="purple"
            disabled={!amplify_enabled}
            onClick={() => {
              if (amplify_enabled)
                call<"scoring/score_update">("scoring/score_update", { update: { alliance: params.alliance, update: "Amplify" } })
            }}
            >
            AMPLIFY
          </Button>
          <br />
          <h5> { score?.derived?.amplified_remaining ? <span className="text-good">AMPLIFIED (T-{(score.derived.amplified_remaining / 1000).toFixed(1)})</span> : <span className="text-muted">Not Amplified</span> } </h5>
        </Col>
      </Row>
    }
  </div>
})