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
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null)

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();
  
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
    </Row>
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
    { score && <Row>
      <Button
        className="btn-block referee-station-score"
        data-score-type="auto_docked"
        data-score-value={score[params.alliance].live.auto_docked}
        onClick={() => call<"scoring/score_update">("scoring/score_update", { update: { alliance: params.alliance, update: { AutoDocked: { docked: !score[params.alliance].live.auto_docked } } } }).then(setScore).catch(addError)}
      >
        {
          score[params.alliance].live.auto_docked ? <React.Fragment> AUTO DOCKED &nbsp; <FontAwesomeIcon icon={faCheck} />  </React.Fragment>
            : <React.Fragment> NO AUTO DOCKED &nbsp; <FontAwesomeIcon icon={faTimes} /> </React.Fragment>
        }
      </Button>
    </Row> }
  </div>
})

export const ENDGAME_MAP: { [K in EndgameType]: string } = {
  None: "None",
  Parked: "ENDGAME Park",
  Docked: "ENDGAME Docked"
};

function RefereeTeamCard({ idx, station, score, update }: { idx: number, station: AllianceStation, score: SnapshotScore, update: (u: ScoreUpdate) => void }) {
  const alliance = station.id.alliance;
  const has_mobility = score.live.mobility[idx];
  
  return withVal(station.team, team => <Col className="referee-station" data-alliance={alliance}>
      <Row>
        <Col className="referee-station-team" md="auto"> { team } </Col>
        <Col>
          <Button
            className="btn-block referee-station-score"
            data-score-type="mobility"
            data-score-value={has_mobility}
            onClick={() => update( { Mobility: { station: idx, crossed: !has_mobility } } )}
          >
            {
              has_mobility ? <React.Fragment> AUTO MOBILITY OK &nbsp; <FontAwesomeIcon icon={faCheck} />  </React.Fragment>
                : <React.Fragment> NO AUTO MOBILITY &nbsp; <FontAwesomeIcon icon={faTimes} /> </React.Fragment>
            }
          </Button>
        </Col>
      </Row>
      <Row>
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
      </Row>
    </Col>) || <Col />;
}
