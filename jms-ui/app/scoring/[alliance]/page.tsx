"use client"

import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions"
import { withValU } from "@/app/support/util";
import { useWebsocket } from "@/app/support/ws-component";
import { Alliance, GamepieceType, LiveScore, Match, MatchScoreSnapshot, SerialisedLoadedMatch, SnapshotScore } from "@/app/ws-schema";
import { Spec } from "immutability-helper";
import React, { useEffect, useState } from "react"
import { Button, Col, Row } from "react-bootstrap";

const ALLOWED_GAMEPIECE_MAP = [
  Array(9).fill(["None", "Cube", "Cone"]),
  [ ["None", "Cone"], ["None", "Cube"], ["None", "Cone"], ["None", "Cone"], ["None", "Cube"], ["None", "Cone"], ["None", "Cone"], ["None", "Cube"], ["None", "Cone"] ],
  [ ["None", "Cone"], ["None", "Cube"], ["None", "Cone"], ["None", "Cone"], ["None", "Cube"], ["None", "Cone"], ["None", "Cone"], ["None", "Cube"], ["None", "Cone"] ],
];

export default withPermission(["Scoring"], function ScorerPanel({ params }: { params: { alliance: Alliance } }) {
  const [ score, setScore ] = useState<SnapshotScore>();
  const [ autoFinalised, setAutoFinalised ] = useState(false);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null)

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();
  
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
      score && <Community2023
        score={score.live}
        alliance={params.alliance}
        onUpdate={(row, col, type) => call<"scoring/score_update">("scoring/score_update", {
          update: { alliance: params.alliance, update: {
            Community: { auto: !autoFinalised, row, col, gamepiece: type }
          } }
        })}
      />
    }
  </div>
})

export function Community2023({ score, alliance, onUpdate }: { score: LiveScore, alliance: Alliance, onUpdate: (row: number, col: number, type: GamepieceType) => void }) {
  let auto = score.community.auto;
  let teleop = score.community.teleop;

  let merged_community: GamepieceType[][] = [ Array(9).fill("None"), Array(9).fill("None"), Array(9).fill("None") ];
  for (let i = 0; i < 3; i++) {
    for (let j = 0; j < 9; j++) {
      if (auto[i][j] != "None") merged_community[i][j] = auto[i][j];
      if (teleop[i][j] != "None") merged_community[i][j] = teleop[i][j];
    }
  }

  return <Row className="scorer-community">
    <Col>
      {
        merged_community.map((row, i) => <Row key={i} className="scorer-community-row">
          {
            row.map((gamepiece, j) => (
              <Col
                key={j}
                className="scorer-community-col"
                data-alliance={alliance}
                data-column={j}
                data-has-auto={auto[i][j] != "None"}
                onClick={() => {
                  let allowed: GamepieceType[] = ALLOWED_GAMEPIECE_MAP[i][j];
                  let current_idx = allowed.findIndex(g => g == gamepiece);
                  let next = (current_idx + 1) % allowed.length;
                  onUpdate(i, j, allowed[next])
                }}
              >
                <div className="scorer-gamepiece" data-gamepiece={gamepiece} />
              </Col>
            ))
          }
        </Row>).reverse()
      }
    </Col>
  </Row>
}