import React from "react";
import JmsWebsocket from "support/ws";
import { ArenaMessageMatch2UI } from "ws-schema";

export default class Timer extends React.PureComponent<{ ws: JmsWebsocket }, ArenaMessageMatch2UI> {
  readonly state: ArenaMessageMatch2UI = { Current: null };
  handle: string = "";

  componentDidMount = () => {
    this.handle = this.props.ws.onMessage<ArenaMessageMatch2UI>(["Arena", "Match", "Current"], msg => this.setState(msg))
  }

  componentWillUnmount = () => {
    this.props.ws.removeHandle(this.handle)
  }

  render() {
    return <div className="timer">
      <div>
        { this.state.Current?.remaining_time?.secs || "---" }
      </div>
    </div>
  }
}