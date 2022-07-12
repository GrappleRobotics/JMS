import moment from "moment";
import React from "react";
import { Col, Navbar } from "react-bootstrap";
import { WebsocketContext, WebsocketContextT } from "support/ws-component";
import { EventDetails, LoadedMatch, SerializedMatch } from "ws-schema";

type BottomNavbarState = {
  event_details?: EventDetails,
  current_match?: LoadedMatch,
  next_match?: SerializedMatch
}

export default class BottomNavbar extends React.Component<{}, BottomNavbarState> {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;

  readonly state: BottomNavbarState = { };
  handles: string[] = [];

  componentDidMount = () => {
    this.handles = [
      this.context.listen<LoadedMatch | null>(["Arena", "Match", "Current"], msg => {
        this.setState({ current_match: msg || undefined })
      }),
      this.context.listen<EventDetails>(["Event", "Details", "Current"], msg => {
        this.setState({ event_details: msg })
      }),
      this.context.listen<SerializedMatch | null>([ "Match", "Next" ], msg => {
        this.setState({ next_match: msg || undefined })
      })
    ]
  }

  componentWillUnmount = () => {
    this.context.unlisten(this.handles);
  }


  getScheduleTimings = () => {
    // @ts-ignore
    let format = (d: moment.Duration) => d.format("d[d] h[h] m[m]", { trim: "both" });

    let nextMatch = this.state.next_match;

    if (nextMatch === undefined || nextMatch === null || !!!nextMatch.start_time)
      return <React.Fragment />;

    let now = moment();
    let match_time = moment.unix(nextMatch.start_time);

    let behind = now > match_time;

    if (behind)
      return <span className={"timekeeper-behind"}>
        { format(moment.duration(now.diff(match_time))) + " BEHIND" }
      </span>
    else
      return <span className={ "timekeeper-ahead" }>
        { format(moment.duration(match_time.diff(now))) + " ahead" }
      </span>
  }

  render() {
    return <Navbar bg="dark" variant="dark" className="flex" fixed="bottom">
      <Col>
        <Navbar.Brand>
          { this.state.current_match?.match_meta?.name || <i> No Match Loaded </i> }
        </Navbar.Brand>
      </Col>
      <Col className="text-center">
        <Navbar.Brand>
          { this.getScheduleTimings() }
        </Navbar.Brand>
      </Col>
      <Col className="text-right">
        <Navbar.Brand>
          <i>
            { this.state.event_details?.event_name }
          </i>
        </Navbar.Brand>
      </Col>
    </Navbar>
  }
}