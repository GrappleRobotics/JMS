import React from "react";
import MatchPlay from "./MatchPlay";
import MatchPreview from "./MatchPreview";

export default class Audience extends React.PureComponent {
  constructor(props) {
    super(props);

    this.props.ws.subscribe("arena", "*");
    this.props.ws.subscribe("event", "*");
  }

  render() {
    let view = window.location.hash.substr(1);

    if (!this.props.arena || !this.props.event)
      return <React.Fragment />

    let { arena, event } = this.props;

    switch (view) {
      case "preview":
        if (arena?.stations && arena?.match?.match)
          return <MatchPreview
            stations={arena.stations}
            match={arena.match.match}
            event={event}
          />
      case "play":
        if (arena?.stations && arena?.match)
          return <MatchPlay
            arena={arena}
            event={event}
          />
      case "field":
      default:
        break;
    }
    return <div className="audience-field" />;
  }
}
