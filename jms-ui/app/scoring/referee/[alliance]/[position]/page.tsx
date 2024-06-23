"use client"
import { withPermission } from "@/app/support/permissions";
import { Alliance, AllianceStation, EndgameType, Match, MatchScoreSnapshot, ScoreUpdate, SerialisedLoadedMatch, SnapshotScore } from "@/app/ws-schema";
import React, { useEffect, useState } from "react";
import { RefereePanelFouls } from "../../referee";
import { useWebsocket } from "@/app/support/ws-component";
import { useToasts } from "@/app/support/errors";
import { Button, Col, Row } from "react-bootstrap";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCheck, faTimes } from "@fortawesome/free-solid-svg-icons";
import { withVal } from "@/app/support/util";
import EnumToggleGroup from "@/app/components/EnumToggleGroup";
import _ from "lodash";

export default withPermission(["Scoring"], function RefereePanel({ params }: { params: { alliance: Alliance, position: string } }) {
  const [ score, setScore ] = useState<MatchScoreSnapshot>();
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ stations, setStations ] = useState<AllianceStation[]>([]);
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

  const [ mode, setMode ] = useState<string>("root");
  
  useEffect(() => {
    let cbs = [
      subscribe<"scoring/current">("scoring/current", setScore),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
      subscribe<"arena/stations">("arena/stations", setStations),
      subscribe<"matches/matches">("matches/matches", setMatches)
    ];
    return () => unsubscribe(cbs);
  }, []);

  return <div className="referee-panel">
    <Row className="mb-3 mt-3">
      <Col>
        <h3 className="mb-0"><i>{ currentMatch ? currentMatch.match_id === "test" ? "Test Match" : matches.find(m => m.id === currentMatch?.match_id)?.name || currentMatch?.match_id : "Waiting for Scorekeeper..." }</i></h3>
        <i className="text-muted"> { params.alliance.toUpperCase() } { params.position.toUpperCase() } Referee </i>
      </Col>
      <Col md="auto">
        <EnumToggleGroup
          name="mode"
          values={[ "root", "stage" ]}
          names={["FOULS & LEAVE", "STAGE & MICS"]}
          value={mode}
          onChange={(m) => setMode(m)}
          variant="outline-secondary"
          variantActive="green"
        />
      </Col>
    </Row>

    {
      mode == "root" ? <React.Fragment>
        { score && <RefereePanelFouls
          score={score}
          onUpdate={update => call<"scoring/score_update">("scoring/score_update", { update: update }).then(setScore).catch(addError)}
          flipped={params.position === "far"}
        /> }
        { score && <Row>
          { stations.filter(s => s.id.alliance === params.alliance).map((stn, i) => (
            <RefereeTeamCard
              key={i}
              idx={i}
              score={score[params.alliance]}
              station={stn}
              update={update => call<"scoring/score_update">("scoring/score_update", { update: { alliance: params.alliance, update: update } }).then(setScore).catch(addError)}
            />
          ))}
        </Row>}
      </React.Fragment> : <React.Fragment>
          { score && <Stage2024 alliance={params.alliance} stations={stations} score={score[params.alliance]} update={update => call<"scoring/score_update">("scoring/score_update", { update: { alliance: params.alliance, update: update } })} /> }
      </React.Fragment>
    }
  </div>
})

function RefereeTeamCard({ idx, station, score, update }: { idx: number, station: AllianceStation, score: SnapshotScore, update: (u: ScoreUpdate) => void }) {
  const alliance = station.id.alliance;
  const has_mobility = score.live.leave[idx];
  
  return withVal(station.team, team => <Col className="referee-station" data-alliance={alliance}>
      <Row>
        <Col className="referee-station-team" md="auto"> { team } </Col>
        <Col>
          <Button
            className="btn-block referee-station-score"
            data-score-type="leave"
            data-score-value={has_mobility}
            onClick={() => update( { Leave: { station: idx, crossed: !has_mobility } } )}
          >
            {
              has_mobility ? <React.Fragment> AUTO LEAVE OK &nbsp; <FontAwesomeIcon icon={faCheck} />  </React.Fragment>
                : <React.Fragment> NO AUTO LEAVE &nbsp; <FontAwesomeIcon icon={faTimes} /> </React.Fragment>
            }
          </Button>
        </Col>
      </Row>
      {/* <Row>
          <Col>
            <EnumToggleGroup
              name={`${team}-endgame`}
              className="referee-station-score"
              data-score-type="endgame"
              data-score-value={score.live.endgame[idx]}
              value={score.live.endgame[idx]}
              values={_.keys(ENDGAME_MAP) as EndgameType[]}
              names={_.values(ENDGAME_MAP)}
              onChange={v => update({ Endgame: { station: idx, endgame: v } })}
              // disabled={!endgame}
            />
          </Col>
      </Row> */}
    </Col>) || <Col />;
}

const STAGE_MAP = [
  "CENTER",
  "LEFT",
  "RIGHT"
]

const STAGE_COLORS = [
  "purple",
  "orange",
  "green"
]

function Stage2024({ alliance, stations, score, update }: { alliance: Alliance, stations: AllianceStation[], score: SnapshotScore, update: (u: ScoreUpdate) => void }) {
  return <Row>
    <Col md="auto">
      <div className="scoring-2024-stage" data-alliance={alliance}>
        <img src={`/img/game/stage_${alliance}.png`}></img>

        {
          [0, 1, 2].map(i => <span className={`scoring-2024-indicator text-${STAGE_COLORS[i]}`} data-index={i}>{ STAGE_MAP[i] }</span>)
        }
      </div>
    </Col>
    <Col md="5">
      <h4> Microphones </h4>
        {[ 0, 1, 2 ].map(i => <Button className="mx-1" variant={score.live.microphones[i] ? STAGE_COLORS[i] : "secondary"} onClick={() => update({ Microphone: { activated: !score.live.microphones[i], stage: i } })}>
          <FontAwesomeIcon icon={ score.live.microphones[i] ? faCheck : faTimes } /> &nbsp; { STAGE_MAP[i] }
        </Button>)}
      <br />
      <br />

      <h4> Traps </h4>
        {[ 0, 1, 2 ].map(i => <Button className="mx-1" variant={score.live.traps[i] ? STAGE_COLORS[i] : "secondary"} onClick={() => update({ Trap: { filled: !score.live.traps[i], stage: i } })}>
          <FontAwesomeIcon icon={ score.live.traps[i] ? faCheck : faTimes } /> &nbsp; { STAGE_MAP[i] }
        </Button>)}
      <br />
      <br />

      <h4> Robots </h4>
        {stations.map((s, i) => <Row>
          { s.team && <Col>
            { s.team }
            <br /> 
            <EnumToggleGroup
              name={`${s.team}-endgame`}
              values={["None", "Parked", { Stage: 0 }, { Stage: 1 }, { Stage: 2 }].map(x => JSON.stringify(x))}
              names={["NONE", "PARK", "CENTER", "LEFT", "RIGHT"]}
              value={JSON.stringify(score.live.endgame[i])}
              onChange={(eg) => update({ Endgame: { endgame: JSON.parse(eg) as EndgameType, station: i } })}
              variant={"outline-secondary"}
              variantActive={
                score.live.endgame[i] == "None" ? "secondary" : 
                score.live.endgame[i] == "Parked" ? "primary" : 
                STAGE_COLORS[(score.live.endgame[i] as any).Stage]
              }
            />
          </Col> }
        </Row>)}
      <br />
      <br />
    </Col>
  </Row>
}