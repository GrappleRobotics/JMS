import React from "react";
import { Button, Container } from "react-bootstrap";
import JmsWebsocket from "support/ws";
import { AllianceStation } from "ws-schema";

type TeamEstopProps = {
  ws: JmsWebsocket,
  station: AllianceStation
}

class TeamEstop extends React.PureComponent<TeamEstopProps> {  
  render() {
    let { station, ws } = this.props;
    return <div className="team-estop">
      <h3> { station.station.alliance } { station.station.station } - { station.team || "No Team" } </h3>
      <br />
      <Button
        size="lg"
        className="estop-all"
        block
        variant="hazard-red-dark"
        disabled={station.estop}
        onClick={() => ws.send({ Arena: { Alliance: { UpdateAlliance: { station: station.station, estop: true } } } })}
      >
        EMERGENCY STOP <br />
        <span className="subtext"> AUTO + TELEOP </span>
      </Button>

      <Button
        className="estop-auto"
        block
        variant="hazard-dark"
        disabled={station.astop || station.estop}
        onClick={() => ws.send({ Arena: { Alliance: { UpdateAlliance: { station: station.station, astop: true } } } })}
      >
        EMERGENCY STOP <br />
        <span className="subtext">AUTO ONLY</span>
      </Button>
    </div>
  }
}

type TeamEstopsState = {
  stations: AllianceStation[]
};

export class TeamEstops extends React.PureComponent<{ ws: JmsWebsocket }, TeamEstopsState> {
  readonly state: TeamEstopsState = { stations: [] };
  handle: string = "";

  componentDidMount = () => {
    this.handle = this.props.ws.onMessage<AllianceStation[]>(["Arena", "Alliance", "CurrentStations"], msg => this.setState({ stations: msg }))
  }

  componentWillUnmount = () => this.props.ws.removeHandle(this.handle);

  render() {
    let stationIdx = parseInt(window.location.hash.substr(1));
    
    return <Container fluid>
      {
        (!isNaN(stationIdx) ? 
          <TeamEstop ws={this.props.ws} station={this.state.stations[stationIdx]} />
          : this.state.stations.map( (s, i) => (
            <Button 
              className="my-3" 
              size="lg" 
              block 
              data-alliance={s.station.alliance.toLowerCase()}
              onClick={() => window.location.hash = "#" + i}
            >
              { s.station.alliance } { s.station.station }
            </Button> 
          )))
      }
    </Container>
  }
}