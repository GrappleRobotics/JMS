"use client"

import { useToasts } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions"
import { withValU } from "@/app/support/util";
import { useWebsocket } from "@/app/support/ws-component";
import { Alliance, LiveScore, Match, MatchScoreSnapshot, SerialisedLoadedMatch, SnapshotScore } from "@/app/ws-schema";
import { Spec } from "immutability-helper";
import React, { useEffect, useState } from "react"
import { Button, Col, Row } from "react-bootstrap";

export default withPermission(["Scoring"], function ScorerPanel({ params }: { params: { alliance: Alliance } }) {
  const [ score, setScore ] = useState<SnapshotScore>();
  const [ autoFinalised, setAutoFinalised ] = useState(false);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null)

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();
  
  useEffect(() => {
    let cbs = [
      subscribe<"scoring/current">("scoring/current", s => setScore(s[params.alliance])),
      subscribe<"arena/current_match">("arena/current_match", (s) => {
        if (s?.state === "Auto" || s?.state === "Waiting")
          setAutoFinalised(false);
        setCurrentMatch(s);
      }),
      subscribe<"matches/matches">("matches/matches", setMatches)
    ];
    return () => unsubscribe(cbs);
  }, []);

  return <div className="scorer-panel">
    <Row className="mb-3 mt-3">
      <Col>
        <h3 className="mb-0"><i>{ currentMatch ? currentMatch.match_id === "test" ? "Test Match" : matches.find(m => m.id === currentMatch?.match_id)?.name || currentMatch?.match_id : "Waiting for Scorekeeper..." }</i></h3>
        <i className="text-muted"> { params.alliance.toUpperCase() } Scorer </i>
      </Col>
      <Col md="auto">
        {
          currentMatch && <Button
            className="scorer-auto-finalise"
            data-finalised={autoFinalised}
            disabled={autoFinalised || (currentMatch.state == "Auto" || currentMatch.state == "Waiting" || currentMatch.state == "Warmup")}
            onClick={() => setAutoFinalised(!autoFinalised)}
          >
            { autoFinalised ? <React.Fragment>AUTO FINALISED</React.Fragment> : <React.Fragment>FINALISE AUTO</React.Fragment> }
          </Button>
        }
      </Col>
    </Row>

    {
      score && <Notes2024
        score={score.live}
        alliance={params.alliance}
        onUpdate={(speaker, amp) => call<"scoring/score_update">("scoring/score_update", {
          update: { alliance: params.alliance, update: {
            Notes: { auto: !autoFinalised, speaker, amp }
          } }
        })}
      />
    }
  </div>
})

export function Notes2024({ score, alliance, onUpdate }: { score: LiveScore, alliance: Alliance, onUpdate: (speaker: number, amp: number) => void }) {
  return <Row>
    <Col>
      <Button
        className="btn-block scorer-button"
        data-button-type="2024-amp"
        data-alliance={alliance}
        variant="orange"
        onClick={() => onUpdate(0, 1)}
      >
        +1 AMP
      </Button>
      <br />
      <span className="text-muted"> AUTO: { score.notes.amp.auto }, TELEOP: { score.notes.amp.teleop } </span>
    </Col>
    <Col>
      <Button
        className="btn-block scorer-button"
        data-button-type="2024-speaker"
        data-alliance={alliance}
        variant={alliance}
        onClick={() => onUpdate(1, 0)}
      >
        +1 SPEAKER
      </Button>
      <br />
      <span className="text-muted"> AUTO: { score.notes.speaker_auto }, TELEOP: { score.notes.speaker_unamped } NORMAL + { score.notes.speaker_amped } AMPED </span>
    </Col>
  </Row>
}