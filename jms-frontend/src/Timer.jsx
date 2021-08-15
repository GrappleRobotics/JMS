import React from "react";

export default class Timer extends React.PureComponent {
  constructor(props) {
    super(props);

    props.ws.subscribe("arena", "match");
  }

  render() {
    return <div className="timer">
      <div>
        { this.props.arena?.match?.remaining_time?.secs }
      </div>
    </div>
  }
}