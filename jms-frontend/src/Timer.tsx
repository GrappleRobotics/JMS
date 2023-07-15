import { WebsocketComponent } from "support/ws-component";
import { ArenaMessageMatch2UI, Duration, LoadedMatch } from "ws-schema";

export default class Timer extends WebsocketComponent<{}, { remaining?: Duration }> {
  readonly state: { remaining?: Duration } = {};

  componentDidMount = () => this.handles = [
    this.listenFn("Arena/Match/Current", (msg?: LoadedMatch) => this.setState({ remaining: msg?.remaining_time }))
  ];

  render() {
    return <div className="timer">
      <div>
        { this.state.remaining?.secs || "---" }
      </div>
    </div>
  }
}