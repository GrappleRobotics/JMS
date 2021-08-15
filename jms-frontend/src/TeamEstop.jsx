import React from "react";
import { Button, Container } from "react-bootstrap";

class TeamEstop extends React.PureComponent {
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
        onClick={() => ws.send("arena", "alliances", "update", {
          station: station.station,
          update: {
            estop: true
          }
        })}
      >
        EMERGENCY STOP
      </Button>

      <Button
        className="estop-auto"
        block
        variant="hazard-dark"
        disabled={station.astop || station.estop}
        onClick={() => ws.send("arena", "alliances", "update", {
          station: station.station,
          update: {
            astop: true
          }
        })}
      >
        EMERGENCY STOP <br />
        <span className="subtext">AUTO ONLY</span>
      </Button>
    </div>
  }
}

export class TeamEstops extends React.PureComponent {
  constructor(props) {
    super(props);

    props.ws.subscribe("arena", "stations");
  }

  render() {
    let stationIdx = parseInt(window.location.hash.substr(1));
    
    return <Container fluid>
      {
        this.props.arena?.stations ? 
          (!isNaN(stationIdx) ? 
            <TeamEstop ws={this.props.ws} station={this.props.arena.stations[stationIdx]} /> 
            : this.props.arena.stations.map((s, i) => (
                <Button
                  className="my-3"
                  // @ts-ignore
                  size="xl"
                  block
                  variant={`alliance-${s.station.alliance.toLowerCase()}`}
                  onClick={() => window.location.hash = "#" + i}
                >
                  { s.station.alliance } { s.station.station }
                </Button>
              )))
          : <React.Fragment />
      }
    </Container>
  }
}