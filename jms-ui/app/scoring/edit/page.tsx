"use client"
import { useToasts } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { Alliance, AllianceStation, CommittedMatchScores, DerivedScore, EndgameType, LiveScore, Match, MatchScore, MatchScoreSnapshot, SerialisedLoadedMatch, SnapshotScore } from "@/app/ws-schema";
import { faCheck, faInfoCircle, faShuffle, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Spec } from "immutability-helper";
import React, { useEffect, useState } from "react";
import { Button, Card, Col, Container, Form, InputGroup, Nav, Row, Table } from "react-bootstrap";
import update from "immutability-helper";
import { ALLIANCES } from "@/app/support/alliances";
import BufferedFormControl from "@/app/components/BufferedFormControl";
import { withConfirm } from "@/app/components/Confirm";
import EnumToggleGroup from "@/app/components/EnumToggleGroup";
import { STAGE_COLORS, STAGE_MAP } from "../referee/[alliance]/[position]/page";

type ModeT = "Live" | "Committed";

export default withPermission(["EditScores"], function EditScores() {
  const [ mode, setMode ] = useState<ModeT>("Live");

  return <Container>
    <Row>
      <Col>
        <h2 className="mt-3 mb-0"> Edit Scores </h2>
      </Col>
      <Col className="mt-4" md="auto">
        <EnumToggleGroup
          name="mode"
          value={mode}
          onChange={setMode}
          values={["Live", "Committed"]}
          variant="secondary"
          size="lg"
        />
      </Col>
    </Row>
    <Row className="mt-3">
      <Col>
        {
          mode == "Committed" ? <EditScoreCommitted /> : <EditScoreLive />
        }
      </Col>
    </Row>
  </Container>
});

function EditScoreLive() {
  const [ score, setScore ] = useState<MatchScore | null>(null);
  const [ newScore, setNewScore ] = useState<MatchScore | null>(null);
  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ stations, setStations ] = useState<AllianceStation[]>([]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

  useEffect(() => {
    let cbs = [
      subscribe<"scoring/current">("scoring/current", s => {
        if (newScore === null)
          setNewScore({ blue: s.blue.live, red: s.red.live });
        setScore({ blue: s.blue.live, red: s.red.live });
      }),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
      subscribe<"matches/matches">("matches/matches", setMatches),
      subscribe<"arena/stations">("arena/stations", setStations),
    ];
    return () => unsubscribe(cbs);
  }, []);

  const match = matches.find(m => m.id === currentMatch?.match_id);
  const upToDate = JSON.stringify(score) == JSON.stringify(newScore);

  return <Container>
    <h3> Edit Live Scores </h3>
    {
      !(currentMatch && newScore && score && match) ? <h4 className="text-danger">No match loaded!</h4> : <React.Fragment>
        <p className="text-muted"><FontAwesomeIcon icon={faInfoCircle} /> &nbsp; You are editing scores for <span className="text-good">{ match.name }</span></p>

        <Row>
          <Col>
            <Button
              variant="good"
              onClick={() => withConfirm(() => {
                call<"scoring/score_full_update">("scoring/score_full_update", { score: newScore })
                  .catch(addError)
              }, undefined, { okVariant: "good" })}
              disabled={upToDate}
            >
              UPDATE SCORES
            </Button>
            &nbsp;
            <Button
              onClick={() => setNewScore(score)}
              disabled={upToDate}
            >
              RESET TO LIVE
            </Button>
          </Col>
        </Row>

        <EditScoresInner
          score={newScore}
          onUpdate={u => setNewScore(update(newScore, u))}
          disabled={false}
          match={match}
          stations={stations}
        />
      </React.Fragment>
    }
  </Container>
}

function EditScoreCommitted() {
  const [ matches, setMatches ] = useState<Match[]>();
  const [ targetMatch, setTargetMatch ] = useState<string>();
  const [ committedScore, setCommittedScore ] = useState<CommittedMatchScores>();
  const [ activeVersion, setActiveVersion ] = useState<number>(0);
  const [ newScore, setNewScore ] = useState<MatchScore>();

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useToasts();

  useEffect(() => {
    let cbs = [
      subscribe<"matches/matches">("matches/matches", setMatches),
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
    <h3> Edit Committed Scores </h3>
    <p className="text-muted"><FontAwesomeIcon icon={faInfoCircle} /> &nbsp; Here you can edit scores for matches before or after they are run. All edits will trigger a recalculation of team rankings if appropriate.</p>
    
    <Row>
      <Col>
        <Form.Select value={targetMatch || ""} onChange={v => setMatch(v.target.value)}>
          <option value="">Select a Match</option>
          {
            matches?.map(m => <option key={m.id} value={m.id}>{ m.name }</option>)
          }
        </Form.Select>
      </Col>
    </Row>
    <br />
    {
      match && (committedScore ? <Row>
        <Col md={2}>
          <Nav variant="pills" className="flex-column">
            <h6 className="text-muted"> Select Version </h6>
            {
              committedScore.scores.map((s, i) => <Nav.Item key={i}>
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
        <h4>This match has not been played yet! Do you want to create a Score Record anyway?</h4>
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
}

function EditScoresInner({ score, onUpdate, disabled, match, stations }: { score: MatchScore, onUpdate: (u: Spec<MatchScore>) => void, disabled: boolean, match: Match, stations?: AllianceStation[] }) {
  const [ derivedScore, setDerivedScore ] = useState<MatchScoreSnapshot>();
  
  const { call } = useWebsocket();
  const { addError } = useToasts();

  useEffect(() => {
    if (score !== undefined)
      call<"scoring/derive_score">("scoring/derive_score", { score: score }).then(setDerivedScore).catch(addError)
  }, [ score ])
  
  return <React.Fragment>
    { score && derivedScore && ALLIANCES.map(alliance => <Row key={alliance as string} className="mt-2">
      <Col>
        <Card className="card-dark" data-alliance={alliance}>
          <Card.Body>
            <Row>
              <Col>
                <h6> { alliance.toUpperCase() } ALLIANCE </h6>
              </Col>
              <Col md="auto">
                { derivedScore[alliance].derived.total_score }pts - +{ derivedScore[alliance].derived.total_rp }RP
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
                    value={score[alliance].penalties.fouls}
                    onUpdate={v => onUpdate({ [alliance]: { penalties: { fouls: { $set: (v as number || 0) } } } })}
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
                    value={score[alliance].penalties.tech_fouls}
                    onUpdate={v => onUpdate({ [alliance]: { penalties: { tech_fouls: { $set: (v as number || 0) } } } })}
                  />
                </InputGroup>
              </Col>
            </Row>
            
            <YearSpecificAllianceScoreEdit
              alliance={alliance}
              live={score[alliance]}
              derived={derivedScore[alliance].derived}
              disabled={disabled}
              onUpdate={s => onUpdate({ [alliance]: s })}
              match={match}
              stations={stations?.filter(s => s.id.alliance == alliance).sort((a, b) => a.id.station - b.id.station)}
            />

            <Row className="mt-2">
              <Col>
                <InputGroup>
                  <InputGroup.Text>ADJUSTMENT</InputGroup.Text>
                  <BufferedFormControl
                    auto
                    type="number"
                    disabled={disabled}
                    value={score[alliance].adjustment}
                    onUpdate={v => onUpdate({ [alliance]: { adjustment: { $set: (v as number || 0) } } })}
                  />
                </InputGroup>
              </Col>
            </Row>
          </Card.Body>
        </Card>
      </Col>
    </Row>) }
  </React.Fragment>
}

function YearSpecificAllianceScoreEdit({ alliance, live, derived, onUpdate, disabled, match, stations }: { alliance: Alliance, live: LiveScore, derived: DerivedScore, onUpdate: (u: Spec<LiveScore>) => void, disabled: boolean, match: Match, stations?: AllianceStation[] }) {
  return <React.Fragment>
    {/* Co-op & Notes */}
    <Row className="mt-2">
      <Col md="auto">
        <Button variant={live.coop ? "good" : "bad"} onClick={() => onUpdate({ coop: { $set: !live.coop } })}>
          <FontAwesomeIcon icon={live.coop ? faCheck : faTimes} /> &nbsp; { live.coop ? "CO-OP" : "NO CO-OP" }
        </Button>
      </Col>
      <Col>
        <InputGroup>
          <InputGroup.Text>AMP Notes (A)</InputGroup.Text>
          <BufferedFormControl
            auto
            type="number"
            disabled={disabled}
            value={live.notes.amp.auto}
            onUpdate={v => onUpdate({ notes: { amp: { auto: { $set: (v as number || 0) } } } })}
          />
        </InputGroup>
      </Col>
      <Col>
        <InputGroup>
          <InputGroup.Text>AMP Notes (T)</InputGroup.Text>
          <BufferedFormControl
            auto
            type="number"
            disabled={disabled}
            value={live.notes.amp.teleop}
            onUpdate={v => onUpdate({ notes: { amp: { teleop: { $set: (v as number || 0) } } } })}
          />
        </InputGroup>
      </Col>
    </Row>
    <Row className="mt-2">
      <Col>
        <InputGroup>
          <InputGroup.Text>Spk Notes (A)</InputGroup.Text>
          <BufferedFormControl
            auto
            type="number"
            disabled={disabled}
            value={live.notes.speaker_auto}
            onUpdate={v => onUpdate({ notes: { speaker_auto: { $set: (v as number || 0) } } })}
          />
        </InputGroup>
      </Col>
      <Col>
        <InputGroup>
          <InputGroup.Text>Spk Notes (T AMPd)</InputGroup.Text>
          <BufferedFormControl
            auto
            type="number"
            disabled={disabled}
            value={live.notes.speaker_amped}
            onUpdate={v => onUpdate({ notes: { speaker_amped: { $set: (v as number || 0) } } })}
          />
        </InputGroup>
      </Col>
      <Col>
        <InputGroup>
          <InputGroup.Text>Spk Notes (T)</InputGroup.Text>
          <BufferedFormControl
            auto
            type="number"
            disabled={disabled}
            value={live.notes.speaker_unamped}
            onUpdate={v => onUpdate({ notes: { speaker_unamped: { $set: (v as number || 0) } } })}
          />
        </InputGroup>
      </Col>
    </Row>

    {/* Teams */}
    <Row className="mt-2">
      {
        (stations ? stations.map(s => s.team) : match[`${alliance}_teams`]).map((t, i) => <Col key={i}> <strong>{  t ? `TEAM ${t}` : `Station ${i + 1}` }</strong> </Col>)
      }
    </Row>
    <Row className="mt-2">
      {
        live.leave.map((mobility, i) => <Col key={i}>
          <Button className="btn-block" variant={mobility ? "success" : "danger"} onClick={() => onUpdate({ leave: { [i]: { $set: !mobility } }})} disabled={disabled}>
            AUTO LEAVE - { mobility ? "OK" : "NOT OK" }
          </Button>
        </Col>)
      }
    </Row>
    <Row className="mt-2">
      {
        live.endgame.map((eg, i) => <Col key={i}>
          <EnumToggleGroup
            name={`${i}-endgame`}
            values={["None", "Parked", { Stage: 0 }, { Stage: 1 }, { Stage: 2 }].map(x => JSON.stringify(x))}
            names={["NONE", "PARK", "CENT.", "LEFT", "RIGHT"]}
            value={JSON.stringify(eg)}
            onChange={(eg2) => onUpdate({ endgame: { [i]: { $set: JSON.parse(eg2) as EndgameType } } })}
            variant={"secondary"}
            variantActive={
              eg == "None" ? "dark" : 
              eg == "Parked" ? "primary" : 
              STAGE_COLORS[(eg as any).Stage]
            }
            size="sm"
          />
        </Col>)
      }
    </Row>

    {/* Stage */}
    <Row className="mt-2">
      <Col>
        <h5> Microphones </h5>
        {[ 0, 1, 2 ].map(i => <Button key={`mic-${i}`} className="mx-1" variant={live.microphones[i] ? STAGE_COLORS[i] : "secondary"} onClick={() => onUpdate({ microphones: { [i]: { $set: !live.microphones[i] } } })}>
          <FontAwesomeIcon icon={ live.microphones[i] ? faCheck : faTimes } /> &nbsp; { STAGE_MAP[i] }
        </Button>)}
      </Col>
      <Col>
        <h5> Traps </h5>
        {[ 0, 1, 2 ].map(i => <Button key={`trap-${i}`} className="mx-1" variant={live.traps[i] ? STAGE_COLORS[i] : "secondary"} onClick={() => onUpdate({ traps: { [i]: { $set: !live.traps[i] } } })}>
          <FontAwesomeIcon icon={ live.traps[i] ? faCheck : faTimes } /> &nbsp; { STAGE_MAP[i] }
        </Button>)}
      </Col>
    </Row>

    {/* Overrides */}
    <Row className="mt-2">
      <Col>
        <Button className="btn-block" variant={live.coop_adjust ? "success" : "danger"} onClick={() => onUpdate({ coop_adjust: { $set: !live.coop_adjust } })} disabled={disabled}>
          CO-OP OVRD { live.coop_adjust ? "TRUE" : "FALSE" }
        </Button>
      </Col>
      <Col>
        <Button className="btn-block" variant={live.melody_adjust ? "success" : "danger"} onClick={() => onUpdate({ melody_adjust: { $set: !live.melody_adjust } })} disabled={disabled}>
          MELODY OVRD { live.melody_adjust ? "TRUE" : "FALSE" }
        </Button>
      </Col>
      <Col>
        <Button className="btn-block" variant={live.ensemble_adjust ? "success" : "danger"} onClick={() => onUpdate({ ensemble_adjust: { $set: !live.ensemble_adjust } })} disabled={disabled}>
          ENSEM. OVRD { live.ensemble_adjust ? "TRUE" : "FALSE" }
        </Button>
      </Col>
    </Row>
  </React.Fragment>
}