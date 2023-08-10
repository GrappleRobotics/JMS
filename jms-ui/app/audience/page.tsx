"use client"
import "./audience.scss";

import { useEffect, useState } from "react";
import { useWebsocket } from "../support/ws-component"
import { AllianceStation, AudienceDisplay, Award, EventDetails, Match, PlayoffAlliance, PlayoffMode, SerialisedLoadedMatch, Team, TeamRanking } from "../ws-schema";
import React from "react";
import { CSSTransition, SwitchTransition, TransitionGroup } from "react-transition-group";
import FieldScene from "./scenes/field";
import MessageScene from "./scenes/message";
import MatchPreviewScene from "./scenes/match-preview";
import PlayoffBracketScene from "./scenes/bracket";
import AllianceSelectionScene from "./scenes/alliance-selections";
import MatchResultsScene from "./scenes/match-results";
import AwardScene from "./scenes/award";

export function withDefaultTransition(key: string, children: React.ReactNode) {
  return <CSSTransition key={key} timeout={500} classNames="audience-scene-anim">
    { children }
  </CSSTransition>
}

export default function AudienceDisplay() {
  const [ eventDetails, setEventDetails ] = useState<EventDetails>();
  const [ audienceDisplay, setAudienceDisplay ] = useState<AudienceDisplay>({ scene: { scene: "Blank" } });

  const [ currentMatch, setCurrentMatch ] = useState<SerialisedLoadedMatch | null>(null);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ teams, setTeams ] = useState<Team[]>([]);
  const [ rankings, setRankings ] = useState<TeamRanking[]>([]);
  const [ stations, setStations ] = useState<AllianceStation[]>([]);
  const [ playoffMode, setPlayoffMode ] = useState<PlayoffMode>();
  const [ nextMatch, setNextMatch ] = useState<Match | null>(null);
  const [ alliances, setAlliances ] = useState<PlayoffAlliance[]>([]);
  const [ awards, setAwards ] = useState<Award[]>([]);

  const { call, subscribe, unsubscribe } = useWebsocket();

  const refreshPlayoffMode = () => {
    call<"matches/get_playoff_mode">("matches/get_playoff_mode", null)
      .then(setPlayoffMode)
  }

  useEffect(() => {
    let cbs = [
      subscribe<"event/details">("event/details", setEventDetails),
      subscribe<"audience/current">("audience/current", setAudienceDisplay),
      subscribe<"arena/current_match">("arena/current_match", setCurrentMatch),
      subscribe<"matches/matches">("matches/matches", (m) => {
        setMatches(m);
        refreshPlayoffMode();
      }),
      subscribe<"matches/next">("matches/next", setNextMatch),
      subscribe<"team/teams">("team/teams", setTeams),
      subscribe<"scoring/rankings">("scoring/rankings", setRankings),
      subscribe<"arena/stations">("arena/stations", setStations),
      subscribe<"alliances/alliances">("alliances/alliances", setAlliances),
      subscribe<"awards/awards">("awards/awards", setAwards),
    ];

    refreshPlayoffMode();

    return () => unsubscribe(cbs)
  }, [])

  useEffect(() => {
    console.log(audienceDisplay);
  }, [ audienceDisplay ]);

  if (!eventDetails) {
    return <React.Fragment />
  }

  const scene = audienceDisplay.scene;

  return <div className="audience-root" style={ { "--chroma-key-colour": eventDetails.av_chroma_key, "--event-colour": eventDetails.av_event_colour } as React.CSSProperties }>
    <SwitchTransition mode="out-in">
      {
        scene.scene === "Blank" ? withDefaultTransition("Blank", <FieldScene />)
        : scene.scene === "CustomMessage" ? withDefaultTransition("CustomMessage", <MessageScene eventDetails={eventDetails} message={scene.params} />)
        : scene.scene === "MatchPreview" ? withDefaultTransition("MatchPreview", <MatchPreviewScene eventDetails={eventDetails} currentMatch={currentMatch} matches={matches} teams={teams} rankings={rankings} stations={stations} />)
        : scene.scene === "MatchResults" ? withDefaultTransition("MatchResults", <MatchResultsScene match_id={scene.params} eventDetails={eventDetails} teams={teams} matches={matches} />)
        : scene.scene === "PlayoffBracket" ? withDefaultTransition("PlayoffBracket", <PlayoffBracketScene eventDetails={eventDetails} matches={matches} teams={teams} playoff_mode={playoffMode?.mode} next_match={nextMatch || undefined} />)
        : scene.scene === "AllianceSelection" ? withDefaultTransition("AllianceSelection", <AllianceSelectionScene eventDetails={eventDetails} alliances={alliances} teams={teams} rankings={rankings} />)
        : scene.scene === "Award" ? withDefaultTransition("Award", <AwardScene award_id={scene.params} eventDetails={eventDetails} teams={teams} awards={awards} />)
        : <React.Fragment />
      }
    </SwitchTransition>
  </div>
}