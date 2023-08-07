"use client"
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { CommittedMatchScores, DerivedScore, LiveScore, Match, MatchScore, MatchScoreSnapshot } from "@/app/ws-schema";
import { faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Spec } from "immutability-helper";
import React, { useEffect, useState } from "react";
import { Button, Card, Col, Container, Form, Nav, Row, Table } from "react-bootstrap";
import update from "immutability-helper";
import { ALLIANCES } from "@/app/support/alliances";

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
              <EditScoresInner score={activeVersion === committedScore.scores.length ? newScore! : committedScore.scores[activeVersion]} onUpdate={u => setNewScore(update(newScore!, u))} disabled={activeVersion !== committedScore.scores.length} />
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

function EditScoresInner({ score, onUpdate, disabled }: { score: MatchScore, onUpdate: (u: Spec<MatchScore>) => void, disabled: boolean }) {
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
        <Card data-alliance={alliance}>
          <Card.Body>
            <h6> { alliance.toUpperCase() } ALLIANCE </h6>
            <YearSpecificAllianceScoreEdit
              live={score[alliance]}
              derived={derivedScore[alliance].derived}
              disabled={disabled}
              onUpdate={s => onUpdate({ [alliance]: s })}
            />
          </Card.Body>
        </Card>
      </Col>
    </Row>) }
  </React.Fragment>
}

function YearSpecificAllianceScoreEdit({ live, derived, onUpdate, disabled }: { live: LiveScore, derived: DerivedScore, onUpdate: (u: Spec<LiveScore>) => void, disabled: boolean }) {
  return <React.Fragment>
    <Row>
      <Col>
        Auto DOCKED?
        <Form.Switch
          disabled={disabled}
          onChange={() => onUpdate({ auto_docked: { $set: !live.auto_docked } })}
          checked={live.auto_docked}
        />
      </Col>
      <Col>
        CS Level (AUTO)
        <Form.Switch
          disabled={disabled}
          onChange={() => onUpdate({ charge_station_level: { auto: { $set: !live.charge_station_level.auto } } })}
          checked={live.charge_station_level.auto}
        />
      </Col>
      <Col>
        CS Level (TELE)
        <Form.Switch
          disabled={disabled}
          onChange={() => onUpdate({ charge_station_level: { teleop: { $set: !live.charge_station_level.teleop } } })}
          checked={live.charge_station_level.teleop}
        />
      </Col>
    </Row>
  </React.Fragment>
}