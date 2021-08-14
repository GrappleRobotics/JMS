import React from "react";
import AllianceSelection from "./AllianceSelection";
import CustomMessage from "./CustomMessage";
import MatchPlay from "./MatchPlay";
import MatchPreview from "./MatchPreview";
import MatchResults from "./MatchResults";

export default class Audience extends React.PureComponent {
  constructor(props) {
    super(props);

    this.props.ws.subscribe("arena", "*");
    this.props.ws.subscribe("event", "*");
    // this.props.ws.subscribe("matches", "*");
  }

  componentDidUpdate(prevProps) {
    const last_match_state = prevProps.arena?.match?.state;
    const match_state = this.props.arena?.match?.state;

    if (last_match_state !== undefined && last_match_state !== match_state) {
      switch (match_state) {
        case "Auto":
          this.audio = new Audio("/sounds/auto.wav");
          this.audio.play();
          break;
        case "Teleop":
          this.audio = new Audio("/sounds/teleop.wav");
          this.audio.play();
          break;
        case "Cooldown":
          this.audio = new Audio("/sounds/match_stop.wav");
          this.audio.play();
          break;
        default:
          break;
      }
    }

    const last_endgame = prevProps.arena?.match?.endgame;
    const endgame = this.props.arena?.match?.endgame;

    if (last_endgame !== undefined && !last_endgame && endgame) {
      this.audio = new Audio("/sounds/endgame.wav");
      this.audio.play();
    }

    const last_arena_state = prevProps.arena?.state?.state;
    const arena_state = this.props.arena?.state?.state;

    if (last_arena_state !== undefined && last_arena_state !== arena_state && arena_state === "Estop") {
      this.audio = new Audio("/sounds/estop.wav");
      this.audio.play();
    }
  }

  render() {
    if (!this.props.arena || !this.props.event)
      return <React.Fragment />
    
    let display = this.props.arena.audience_display;
    let { arena, event } = this.props;

    switch (display?.scene) {
      case "MatchPreview":
        if (arena?.stations && arena?.match?.match)
          return <MatchPreview
            stations={arena.stations}
            match={arena.match.match}
            event={event}
          />
        break;
      case "MatchPlay":
        if (arena?.stations && arena?.match)
          return <MatchPlay
            arena={arena}
            event={event}
          />
        break;
      case "MatchResults":
        if (display.params)
          return <MatchResults
            match={display.params}
            event={event}
          />
        break;
      case "AllianceSelection":
        if (event.alliances)
          return <AllianceSelection
            event={event}
          />
        break;
      case "CustomMessage":
        if (display.params)
          return <CustomMessage
            event={event}
            msg={display.params}
          />
        break;
      case "Field":
      default:
        break;
    }
    return <div className="audience-field" />;
  }
}
