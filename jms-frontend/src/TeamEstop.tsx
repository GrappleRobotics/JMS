import React from "react";
import { Button, Container } from "react-bootstrap";
import { WebsocketComponent } from "support/ws-component";
import { AllianceStation } from "ws-schema";

type TeamEstopProps = {
  station: AllianceStation,
  onTrigger: (which: "astop" | "estop") => void
}

// TODO: Guard with an "ARE YOU SURE?"
class TeamEstop extends React.PureComponent<TeamEstopProps> {  
  render() {
    let { station, onTrigger } = this.props;
    return <div className="team-estop">
      <h3> { station.station.alliance } { station.station.station } - { station.team || "No Team" } </h3>
      <br />
      <Button
        size="lg"
        className="estop-all"
        variant="hazard-red-dark"
        disabled={station.estop}
        onClick={() => onTrigger("estop")}
      >
        EMERGENCY STOP <br />
        <span className="subtext"> AUTO + TELEOP </span>
      </Button>

      <Button
        className="estop-auto"
        variant="hazard-dark"
        disabled={station.astop || station.estop}
        onClick={() => onTrigger("astop")}
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

export class TeamEstops extends WebsocketComponent<{}, TeamEstopsState> {
  readonly state: TeamEstopsState = { stations: [] };

  componentDidMount = () => this.handles = [
    this.listen("Arena/Alliance/CurrentStations", "stations")
  ]

  render() {
    let stationIdx = parseInt(window.location.hash.substr(1));
    
    return <Container fluid>
      {
        (!isNaN(stationIdx) ? 
          <TeamEstop 
            station={this.state.stations[stationIdx]}
            onTrigger={which => this.send({
              Arena: { Alliance: { UpdateAlliance: {
                [which]: true,
                station: this.state.stations[stationIdx].station
              } } }
            })}
          />
          : this.state.stations.map( (s, i) => (
            <Button 
              className="my-3 btn-block" 
              size="lg" 
              variant={`${s.station.alliance.toLowerCase()}`}
              onClick={() => window.location.hash = "#" + i}
            >
              { s.station.alliance } { s.station.station }
            </Button> 
          )))
      }
    </Container>
  }
}