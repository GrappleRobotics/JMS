import moment from "moment";
import React from "react";
import { Col, Navbar } from "react-bootstrap";

export default class BottomNavbar extends React.Component {

  allMatches = () => {
    if (!this.props.matches)
      return [];
    return Object.values(this.props.matches).flatMap(x => x.matches || []);
  }

  getScheduleTimings = () => {
    let format = d => d.format("d[d] h[h] m[m]", { trim: "both" });

    let nextMatch = this.allMatches().find(m => !m.played && m.id != this.props.arena?.match?.match?.id);

    if (nextMatch === undefined || nextMatch === null)
      return <React.Fragment />;

    let now = moment();
    let match_time = moment.unix(nextMatch.time);

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
    let { arena, event } = this.props;
    return <Navbar bg="dark" variant="dark" className="flex" fixed="bottom">
      <Col>
        <Navbar.Brand>
          { arena?.match?.match?.name || <i> No Match Loaded </i> }
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
            { event?.event_name }
          </i>
        </Navbar.Brand>
      </Col>
    </Navbar>
  }
}