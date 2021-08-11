import React from "react";
import MatchPlay from "./MatchPlay";
import MatchPreview from "./MatchPreview";
import MatchResults from "./MatchResults";

export default class Audience extends React.PureComponent {
  constructor(props) {
    super(props);

    this.props.ws.subscribe("arena", "*");
    this.props.ws.subscribe("event", "*");
    this.props.ws.subscribe("matches", "*");
  }

  render() {
    let view = window.location.hash.substr(1);

    if (!this.props.arena || !this.props.event || !this.props.matches)
      return <React.Fragment />

    let { arena, event, matches } = this.props;

    switch (view) {
      case "preview":
        if (arena?.stations && arena?.match?.match)
          return <MatchPreview
            stations={arena.stations}
            match={arena.match.match}
            event={event}
          />
        break;
      case "play":
        if (arena?.stations && arena?.match)
          return <MatchPlay
            arena={arena}
            event={event}
          />
        break;
      case "results":
        if (matches.last)
          return <MatchResults
            match={matches.last}
            event={event}
          />
        break;
      case "field":
      default:
        break;
    }
    return <div className="audience-field" />;
  }
}
