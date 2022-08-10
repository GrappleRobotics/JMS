import React from "react";
import { WebsocketComponent } from "support/ws-component";
import { AudienceDisplay, EventDetails } from "ws-schema";
import AudienceSceneAllianceSelection from "./AllianceSelection";
import AudienceSceneAward from "./Award";
import { AudienceSceneField } from "./BaseAudienceScene";
import AudienceSceneCustomMessage from "./CustomMessage";
import AudienceSceneMatchPlay from "./MatchPlay";
import AudienceSceneMatchPreview from "./MatchPreview";
import AudienceSceneMatchResults from "./MatchResults";

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
      return <React.Fragment>
        <AudienceSceneField details={event_details} props={scene.scene === "Field" ? {} : undefined} />
        <AudienceSceneCustomMessage details={event_details} props={scene.scene === "CustomMessage" ? { msg: scene.params } : undefined} />
        <AudienceSceneAward details={event_details} props={scene.scene === "Award" ? scene.params : undefined} />
        <AudienceSceneMatchPreview details={event_details} props={scene.scene === "MatchPreview" ? {} : undefined} />
        <AudienceSceneMatchPlay details={event_details} props={scene.scene === "MatchPlay" ? {} : undefined} />
        <AudienceSceneMatchResults details={event_details} props={scene.scene === "MatchResults" ? { match: scene.params } : undefined} />
        <AudienceSceneAllianceSelection details={event_details} props={scene.scene === "AllianceSelection" ? {} : undefined} />
    </React.Fragment>
    } else {
      return <React.Fragment />
    }
  }
}

// export default class Audience extends React.PureComponent {
//   constructor(props) {
//     super(props);

//     // this.props.ws.subscribe("arena", "*");
//     // this.props.ws.subscribe("event", "*");
//   }

//   componentDidUpdate(prevProps) {
//     const last_match_state = prevProps.arena?.match?.state;
//     const match_state = this.props.arena?.match?.state;

//     if (last_match_state !== undefined && last_match_state !== match_state) {
//       switch (match_state) {
//         case "Auto":
//           this.audio = new Audio("/sounds/auto.wav");
//           this.audio.play();
//           break;
//         case "Teleop":
//           this.audio = new Audio("/sounds/teleop.wav");
//           this.audio.play();
//           break;
//         case "Cooldown":
//           this.audio = new Audio("/sounds/match_stop.wav");
//           this.audio.play();
//           break;
//         default:
//           break;
//       }
//     }

//     const last_endgame = prevProps.arena?.match?.endgame;
//     const endgame = this.props.arena?.match?.endgame;

//     if (last_endgame !== undefined && !last_endgame && endgame) {
//       this.audio = new Audio("/sounds/endgame.wav");
//       this.audio.play();
//     }

//     const last_arena_state = prevProps.arena?.state?.state;
//     const arena_state = this.props.arena?.state?.state;

//     if (last_arena_state !== undefined && last_arena_state !== arena_state && arena_state === "Estop") {
//       this.audio = new Audio("/sounds/estop.wav");
//       this.audio.play();
//     }
//   }

//   render() {
//     if (!this.props.arena || !this.props.event)
//       return <React.Fragment />
    
//     let display = this.props.arena.audience_display;
//     let { arena, event } = this.props;

//     switch (display?.scene) {
//       case "MatchPreview":
//         if (arena?.stations && arena?.match?.match)
//           return <MatchPreview
//             stations={arena.stations}
//             match={arena.match.match}
//             event={event}
//           />
//         break;
//       case "MatchPlay":
//         if (arena?.stations && arena?.match)
//           return <MatchPlay
//             arena={arena}
//             event={event}
//           />
//         break;
//       case "MatchResults":
//         if (display.params)
//           return <MatchResults
//             match={display.params}
//             event={event}
//           />
//         break;
//       case "AllianceSelection":
//         if (event.alliances && event.rankings)
//           return <AllianceSelection
//             event={event}
//           />
//         break;
//       case "CustomMessage":
//         if (display.params)
//           return <CustomMessage
//             event={event}
//             msg={display.params}
//           />
//         break;
//       case "Award":
//         if (display.params)
//           return <AudienceAward
//             event={event}
//             award={display.params}
//           />
//       case "Field":
//       default:
//         break;
//     }
//     return <div className="audience-field" />;
//   }
// }
