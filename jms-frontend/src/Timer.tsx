import React from "react";
import JmsWebsocket from "support/ws";
import { WebsocketContext, WebsocketContextT } from "support/ws-component";
import { ArenaMessageMatch2UI } from "ws-schema";

export default class Timer extends React.PureComponent<{}, ArenaMessageMatch2UI> {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;

  readonly state: ArenaMessageMatch2UI = { Current: null };
  handle: string = "";

  componentDidMount = () => {
    this.handle = this.context.listen<ArenaMessageMatch2UI>(["Arena", "Match", "Current"], msg => this.setState(msg))
  }

  componentWillUnmount = () => {
    this.context.unlisten([this.handle])
  }

  render() {
    return <div className="timer">
      <div>
        { this.state.Current?.remaining_time?.secs || "---" }
      </div>
    </div>
  }
}