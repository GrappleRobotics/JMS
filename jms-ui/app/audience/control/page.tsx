"use client"
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import UserPage from "@/app/userpage";
import { AudienceDisplay, AudienceDisplayScene, AudienceDisplaySound, Award, CommittedMatchScores, Match } from "@/app/ws-schema";
import React, { useEffect, useState } from "react";
import { Button } from "react-bootstrap";

const SOUNDS: { id: AudienceDisplaySound, name: string }[] = [
  { id: "AutoStart",    name: "Match Start" },
  { id: "TeleopStart",  name: "Teleop Start" },
  { id: "Endgame",      name: "Endgame Start" },
  { id: "MatchStop",    name: "Match End" },
  { id: "Estop",        name: "E-Stop" },
];

export default withPermission(["ManageAudience"], function AudienceDisplayControl() {
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ lastScores, setLastScores ] = useState<CommittedMatchScores | null>(null);
  const [ awards, setAwards ] = useState<Award[]>([]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    let cbs = [
      subscribe<"scoring/latest_scores">("scoring/latest_scores", setLastScores),
      subscribe<"matches/matches">("matches/matches", setMatches),
      subscribe<"awards/awards">("awards/awards", setAwards),
    ];
    return () => unsubscribe(cbs);
  }, []);

  const setScene = (scene: AudienceDisplayScene) => call<"audience/set">("audience/set", { scene }).catch(addError);
  const play = (sound: AudienceDisplaySound) => call<"audience/play_sound">("audience/play_sound", { sound }).catch(addError);

  const lastMatchName = lastScores ? matches.find(x => x.id === lastScores.match_id)?.name || lastScores.match_id : undefined;

  return <UserPage container>
    <div className="mt-2">
      <h3> Audience Control </h3>
      <p> If displays are not yet ready to display data (e.g. match is not loaded), displays will default to a blank 
        field view until data is ready. </p>
      
      <h4> General Purpose </h4>
      <Button onClick={() => setScene({ scene: "Blank" })}> Blank </Button> &nbsp;
      <Button onClick={() => setScene({ scene: "CustomMessage", params: "Hello World!" })} variant="orange"> Custom Message </Button> &nbsp;

      <h4 className="mt-3"> Matches </h4>
      <Button onClick={() => setScene({ scene: "MatchPreview" })} variant="orange"> Match Preview </Button> &nbsp;
      <Button onClick={() => setScene({ scene: "MatchPlay" })} variant="purple"> Match Play </Button> &nbsp;
      {
        lastScores && <Button onClick={() => setScene({ scene: "MatchResults", params: lastScores.match_id })} variant="success"> Match Results (Latest - { lastMatchName }) </Button>
      }

      <h4 className="mt-3"> Playoffs </h4>
      <Button onClick={() => setScene({ scene: "AllianceSelection" })}> Alliance Selection </Button> &nbsp;
      <Button onClick={() => setScene({ scene: "PlayoffBracket" })} variant="orange"> Playoff Bracket </Button> &nbsp;

      <h4 className="mt-3"> Awards </h4>
      {
        awards.map(award => <Button className="mx-1" key={award.id} variant="gold" onClick={() => setScene({ scene: "Award", params: award.id })}>
          { award.name }
        </Button>)
      }

      <hr />
      <h4 className="mt-3"> Sounds </h4>
      <Button onClick={() => play("AutoStart")} variant="success"> AUTO </Button> &nbsp;
      <Button onClick={() => play("TeleopStart")} variant="orange"> TELEOP </Button> &nbsp;
      <Button onClick={() => play("Endgame")} variant="purple"> ENDGAME </Button> &nbsp;
      <Button onClick={() => play("MatchStop")} variant="secondary"> MATCH STOP </Button> &nbsp;
      <Button onClick={() => play("Estop")} variant="estop"> E-STOP </Button> &nbsp;
    </div>
  </UserPage>
})