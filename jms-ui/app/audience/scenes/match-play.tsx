import { ALLIANCES, otherAlliance } from "@/app/support/alliances";
import { usePrevious, withValU } from "@/app/support/util";
import { AllianceStation, ArenaState, EventDetails, Match, MatchScoreSnapshot, SerialisedLoadedMatch, Team } from "@/app/ws-schema";
import { faLink, faMusic } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React, { useEffect } from "react";
import { Col, Row } from "react-bootstrap";
import { SwitchTransition, CSSTransition, TransitionGroup } from "react-transition-group";
import { playSound } from "../utils";

interface MatchPreviewProps {
  eventDetails: EventDetails,
  currentMatch: SerialisedLoadedMatch | null,
  matches: Match[],
  teams: Team[],
  stations: AllianceStation[],
  arenaState?: ArenaState,
  currentScore?: MatchScoreSnapshot
}

export default function MatchPlayScene({ eventDetails, currentMatch, matches, teams, stations, currentScore, arenaState }: MatchPreviewProps) {
  const match = matches.find(m => m.id === currentMatch?.match_id);
  const lastMatchState = usePrevious(currentMatch?.state);
  const endgame = currentMatch?.endgame;
  const lastEndgame = usePrevious(endgame);

  useEffect(() => {
    if (currentMatch && lastMatchState && currentMatch.state !== lastMatchState) {
      if (currentMatch.state === "Auto") {
        playSound("AutoStart");
      } else if (currentMatch.state === "Pause") {
        playSound("MatchStop");
      } else if (currentMatch.state === "Teleop") {
        playSound("TeleopStart");
      } else if (currentMatch.state === "Cooldown") {
        playSound("MatchStop");
      }
    }

    if (endgame && lastEndgame !== undefined && !lastEndgame) {
      playSound("Endgame");
    }
  }, [ currentMatch, endgame ])

  return <TransitionGroup>
    { currentMatch && <CSSTransition key={"audience-play"} timeout={500} classNames="audience-scene-anim">
      <div>
        <div className="audience-play">
          <div className="audience-play-timer-block" data-arena-state={arenaState?.state}>
            <Row className="audience-play-match">
              <Col>
                { match?.name || currentMatch?.match_id }
              </Col>
            </Row>
            <Row className="audience-play-timer">
              <Col>
                { currentMatch && Math.ceil(currentMatch.remaining / 1000) }
              </Col>
            </Row>
            <Row className="audience-play-mode">
              <Col>
                { arenaState?.state === "MatchPlay" ? currentMatch?.state : arenaState?.state === "Estop" ? "EMERGENCY STOP" : arenaState?.state === "MatchComplete" ? "Match Complete" : "Match Waiting" }
              </Col>
            </Row>
          </div>

          {
            ALLIANCES.map((alliance, i) => {
              const derived = currentScore?.[alliance]?.derived;
              const other = currentScore?.[otherAlliance(alliance)]?.derived;

              const playoffAlliance = match?.[`${alliance}_alliance`];

              return <React.Fragment key={i}>
                <div className="audience-play-score" data-alliance={alliance}>
                  <Row>
                    <Col>
                      { derived?.total_score || 0 }
                    </Col>
                  </Row>
                  { match?.match_type === "Qualification" && withValU(derived?.total_bonus_rp, brp => brp > 0 && <Row className="audience-play-score-bonus-rp">
                    <Col>
                      +{ brp } RP
                    </Col>
                  </Row>) }
                  <Row className="audience-play-score-links">
                    <Col>
                      <FontAwesomeIcon icon={faMusic} /> &nbsp; { derived?.notes.total_count || 0 } / { derived?.melody_threshold }
                    </Col>
                  </Row>
                </div>

                <div className="audience-play-teams" data-alliance={alliance}>
                  {
                    stations.filter(s => s.id.alliance === alliance).map((stn, j) => (
                      <Row key={j} className="audience-play-team" data-alliance={stn.id.alliance} data-station={stn.id.station} data-estop={stn.estop} data-bypass={stn.bypass}>
                        <Col> { teams.find(t => t.number === stn.team)?.display_number || stn.team } </Col>
                      </Row>
                    ))
                  }
                </div>

                {
                  playoffAlliance && <div className="audience-play-alliance" data-alliance={alliance}> Alliance { playoffAlliance } </div>
                }
              </React.Fragment>
            })
          }
        </div>
      </div>
    </CSSTransition> }
  </TransitionGroup>
  
}