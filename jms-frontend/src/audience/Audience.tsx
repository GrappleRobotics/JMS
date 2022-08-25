import React from "react";
import { CSSTransition, TransitionGroup } from "react-transition-group";
import { WebsocketComponent } from "support/ws-component";
import { AudienceDisplay, EventDetails } from "ws-schema";
import AudienceSceneAllianceSelection from "./AllianceSelection";
import AudienceSceneAward from "./Award";
import { AudienceSceneField } from "./BaseAudienceScene";
import AudienceSceneCustomMessage from "./CustomMessage";
import AudienceSceneMatchPlay from "./MatchPlay";
import AudienceSceneMatchPreview from "./MatchPreview";
import AudienceSceneMatchResults from "./MatchResults";
import AudienceScenePlayoffBracket from "./PlayoffBracket";

type AudienceState = {
  scene?: AudienceDisplay,
  event_details?: EventDetails
}

export default class Audience extends WebsocketComponent<{}, AudienceState> {
  readonly state: AudienceState = {};

  componentDidMount = () => this.handles = [
    this.listen("Arena/AudienceDisplay/Current", "scene"),
    this.listen("Event/Details/Current", "event_details")
  ];

  render() {
    const { scene, event_details } = this.state;
    
    if (scene != null && event_details != null) {
      // Render all, but only one is active at a time
      // This way, each element keeps its own listen  so we don't have to wait for websocket updates after mount
      return <div className="audience-root" style={ { "--chroma-key-colour": event_details.av_chroma_key, "--event-colour": event_details.av_event_colour } as React.CSSProperties }>
        {/* <TransitionGroup> */}
          <AudienceSceneField details={event_details} props={scene.scene === "Field" ? {} : undefined} />
          <AudienceSceneCustomMessage details={event_details} props={scene.scene === "CustomMessage" ? { msg: scene.params } : undefined} />
          <AudienceSceneAward details={event_details} props={scene.scene === "Award" ? scene.params : undefined} />
          <AudienceSceneMatchPreview details={event_details} props={scene.scene === "MatchPreview" ? {} : undefined} />
          <AudienceSceneMatchPlay details={event_details} props={scene.scene === "MatchPlay" ? {} : undefined} />
          <AudienceSceneMatchResults details={event_details} props={scene.scene === "MatchResults" ? { match: scene.params } : undefined} />
          <AudienceSceneAllianceSelection details={event_details} props={scene.scene === "AllianceSelection" ? {} : undefined} />
          <AudienceScenePlayoffBracket details={event_details} props={scene.scene === "PlayoffBracket" ? {} : undefined} />
        {/* </TransitionGroup> */}
    </div>
    } else {
      return <React.Fragment />
    }
  }
}
