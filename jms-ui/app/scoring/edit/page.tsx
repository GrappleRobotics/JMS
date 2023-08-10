"use client"
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { Alliance, CommittedMatchScores, DerivedScore, EndgameType, LiveScore, Match, MatchScore, MatchScoreSnapshot } from "@/app/ws-schema";
import { faInfoCircle, faShuffle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Spec } from "immutability-helper";
import React, { useEffect, useState } from "react";
import { Button, Card, Col, Container, Form, InputGroup, Nav, Row, Table } from "react-bootstrap";
import update from "immutability-helper";
import { ALLIANCES } from "@/app/support/alliances";
import BufferedFormControl from "@/app/components/BufferedFormControl";
import { ENDGAME_MAP } from "../referee/[alliance]/[position]/page";
import { Community2023 } from "../[alliance]/page";
import { withConfirm } from "@/app/components/Confirm";

export default withPermission(["EditScores"], function EditScores() {
  const [ matches, setMatches ] = useState<Match[]>();
  const [ targetMatch, setTargetMatch ] = useState<string>();
  const [ committedScore, setCommittedScore ] = useState<CommittedMatchScores>();
  const [ activeVersion, setActiveVersion ] = useState<number>(0);
  const [ newScore, setNewScore ] = useState<MatchScore>();

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    let cbs = [
      subscribe<"matches/matches">("matches/matches", setMatches)
    ];
    return () => unsubscribe(cbs);
  }, [])

  const match = targetMatch ? matches?.find(x => x.id === targetMatch) : undefined;
  const setMatch = (match: string) => {
    if (match === "") {
      setTargetMatch(undefined);
      setCommittedScore(undefined);
    } else {
      setTargetMatch(match);
      call<"scoring/get_committed">("scoring/get_committed", { match_id: match })
        .then(s => {
          setCommittedScore(s);
          setActiveVersion(s.scores.length);
          if (s.scores.length === 0)
            call<"scoring/get_default_scores">("scoring/get_default_scores", null).then(setNewScore).catch(addError)
          else 
            setNewScore(s.scores[s.scores.length - 1]);
        })
        .catch((e) => setCommittedScore(undefined))
    }
  };

  return <Container>
    <h2 className="mt-3 mb-0"> Edit Scores </h2>
    <p className="text-muted"><FontAwesomeIcon icon={faInfoCircle} /> &nbsp; Here you can edit scores for matches before or after they are run. All edits will trigger a recalculation of team rankings if appropriate.</p>
    
    <Row>
      <Col>
        <Form.Select value={targetMatch || ""} onChange={v => setMatch(v.target.value)}>
          <option value="">Select a Match</option>
          {
            matches?.map(m => <option value={m.id}>{ m.name }</option>)
          }
        </Form.Select>
      </Col>
    </Row>
    <br />
    {
      match && (committedScore ? <Row>
        <Col md={3}>
          <Nav variant="pills" className="flex-column">
            <h6 className="text-muted"> Select Version </h6>
            {
              committedScore.scores.map((s, i) => <Nav.Item>
                <Nav.Link className="edit-scores-version-link" data-active={activeVersion === i} onClick={() => setActiveVersion(i)}> Version { i + 1 } </Nav.Link>
              </Nav.Item>)
            }
            <Nav.Item>
              <Nav.Link className="edit-scores-version-link" data-active={activeVersion === committedScore.scores.length} onClick={() => setActiveVersion(committedScore.scores.length)}> (New Version) </Nav.Link>
            </Nav.Item>
          </Nav>
        </Col>
        <Col>
          <Card>
            <Card.Body>
              <h4> { match.name } - Version { activeVersion + 1 } </h4>
              <Button
                variant="danger"
                onClick={() => withConfirm(() => {
                  call<"scoring/push_committed_score">("scoring/push_committed_score", { match_id: match.id, score: newScore! })
                    .then(s => setCommittedScore(s))
                    .catch(addError)
                })}
              >
                PUSH SCORES
              </Button>
              <br /> <br />
              <EditScoresInner
                score={activeVersion === committedScore.scores.length ? newScore! : committedScore.scores[activeVersion]}
                onUpdate={u => setNewScore(update(newScore!, u))}
                disabled={activeVersion !== committedScore.scores.length}
                match={match}
              />
            </Card.Body>
          </Card>
        </Col>
      </Row> : <React.Fragment>
        <h4>This match hasn't been played yet! Do you want to create a Score Record anyway?</h4>
        <Button
          size="lg"
          variant="success"
          onClick={() => call<"scoring/new_committed_record">("scoring/new_committed_record", { match_id: match.id }).then(setCommittedScore).catch(addError)}
        >
          Create Score Record
        </Button>
      </React.Fragment>)
    }
  </Container>
})

function EditScoresInner({ score, onUpdate, disabled, match }: { score: MatchScore, onUpdate: (u: Spec<MatchScore>) => void, disabled: boolean, match: Match }) {
  const [ derivedScore, setDerivedScore ] = useState<MatchScoreSnapshot>();
  
  const { call } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    if (score !== undefined)
      call<"scoring/derive_score">("scoring/derive_score", { score: score }).then(setDerivedScore).catch(addError)
  }, [ score ])
  
  return <React.Fragment>
    { score && derivedScore && ALLIANCES.map(alliance => <Row className="mt-2">
      <Col>
        <Card className="card-dark" data-alliance={alliance}>
          <Card.Body>
            <YearSpecificAllianceScoreEdit
              alliance={alliance}
              live={score[alliance]}
              derived={derivedScore[alliance].derived}
              disabled={disabled}
              onUpdate={s => onUpdate({ [alliance]: s })}
              match={match}
            />
          </Card.Body>
        </Card>
      </Col>
    </Row>) }
  </React.Fragment>
}

function YearSpecificAllianceScoreEdit({ alliance, live, derived, onUpdate, disabled, match }: { alliance: Alliance, live: LiveScore, derived: DerivedScore, onUpdate: (u: Spec<LiveScore>) => void, disabled: boolean, match: Match }) {
  const [ communityAuto, setCommunityAuto ] = useState<boolean>(true);

  return <React.Fragment>
    <Row>
      <Col>
        <h6> { alliance.toUpperCase() } ALLIANCE </h6>
      </Col>
      <Col md="auto">
        { derived.total_score }pts - +{ derived.total_rp }RP
      </Col>
    </Row>
    <Row className="mt-2">
      <Col>
        <InputGroup>
          <InputGroup.Text>FOULS</InputGroup.Text>
          <BufferedFormControl
            auto
            type="number"
            disabled={disabled}
            value={live.penalties.fouls}
            onUpdate={v => onUpdate({ penalties: { fouls: { $set: (v as number || 0) } } })}
          />
        </InputGroup>
      </Col>
      <Col>
        <InputGroup>
          <InputGroup.Text>TECH FOULS</InputGroup.Text>
          <BufferedFormControl
            auto
            type="number"
            disabled={disabled}
            value={live.penalties.tech_fouls}
            onUpdate={v => onUpdate({ penalties: { tech_fouls: { $set: (v as number || 0) } } })}
          />
        </InputGroup>
      </Col>
    </Row>
    <Row className="mt-2">
      <Col>
        <Button className="btn-block" variant={live.auto_docked ? "success" : "danger"} onClick={() => onUpdate({ auto_docked: { $set: !live.auto_docked } })} disabled={disabled}>
          AUTO DOCKED { live.auto_docked ? "OK" : "NOT OK" }
        </Button>
      </Col>
      <Col>
        <Button className="btn-block" variant={live.charge_station_level.auto ? "success" : "danger"} onClick={() => onUpdate({ charge_station_level: { auto: { $set: !live.charge_station_level.auto } }})} disabled={disabled}>
          CS AUTO - { live.charge_station_level.auto ? "LEVEL" : "NOT LEVEL" }
        </Button>
      </Col>
      <Col>
        <Button className="btn-block" variant={live.charge_station_level.teleop ? "success" : "danger"} onClick={() => onUpdate({ charge_station_level: { teleop: { $set: !live.charge_station_level.teleop } }})} disabled={disabled}>
          CS TELEOP - { live.charge_station_level.teleop ? "LEVEL" : "NOT LEVEL" }
        </Button>
      </Col>
    </Row>

    {/* Community */}
    <Row className="mt-2">
      <Col md="auto">
        <strong>Community ({ communityAuto ? "AUTO" : "TELEOP" })</strong>
      </Col>
      <Col>
        <Button size="sm" onClick={() => setCommunityAuto(!communityAuto)}>
          <FontAwesomeIcon icon={faShuffle} />
        </Button>
      </Col>
    </Row>
    <Row>
      <Col style={{ fontSize: '0.5em' }}>
        <Community2023
          score={live}
          alliance={alliance}
          onUpdate={(row, col, type) => onUpdate({ community: { [communityAuto ? "auto" : "teleop"]: { [row]: { [col]: { $set: type } } } } })}
        />
      </Col>
    </Row>

    {/* Teams */}
    <Row className="mt-2">
      {
        match[`${alliance}_teams`].map((t, i) => <Col> <strong>{  t ? `TEAM ${t}` : `Station ${i + 1}` }</strong> </Col>)
      }
    </Row>
    <Row className="mt-2">
      {
        live.mobility.map((mobility, i) => <Col>
          <Button className="btn-block" variant={mobility ? "success" : "danger"} onClick={() => onUpdate({ mobility: { [i]: { $set: !mobility } }})} disabled={disabled}>
            MOBILITY - { mobility ? "OK" : "NOT OK" }
          </Button>
        </Col>)
      }
    </Row>
    <Row className="mt-2">
      {
        live.endgame.map((eg, i) => <Col>
          <Form.Select value={eg} onChange={v => onUpdate({ endgame: { [i]: { $set: v.target.value as EndgameType } } })} disabled={disabled}>
            {
              Object.keys(ENDGAME_MAP).map(egt => <option value={egt}>{ (ENDGAME_MAP as any)[egt] }</option>)
            }
          </Form.Select>
        </Col>)
      }
    </Row>
  </React.Fragment>
}