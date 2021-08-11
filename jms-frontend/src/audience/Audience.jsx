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

  render() {
    if (!this.props.arena || !this.props.event)
      return <React.Fragment />
    
    let display = this.props.arena.audience_display;
    let { arena, event } = this.props;

    switch (display.scene) {
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
