import { WebsocketComponent } from "support/ws-component";
import { ArenaMessageMatch2UI, Duration } from "ws-schema";

export default class Timer extends WebsocketComponent<{}, { remaining?: Duration }> {

  componentDidMount = () => this.handles = [
    this.listenFn("Arena/Match/Current", (msg: ArenaMessageMatch2UI["Current"]) => this.setState({ remaining: msg?.remaining_time }))
  ];

  render() {
    return <div className="timer">
      <div>
        { this.state.remaining?.secs || "---" }
      </div>
    </div>
  }
}