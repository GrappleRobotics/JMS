import AllianceDash from "./Alliance";
import MatchFlow from "./MatchFlow";
import React from "react";
import { Button, Col, Container, Row } from "react-bootstrap";
import MatchScheduleView from "./MatchScheduleView";
import JmsWebsocket from "support/ws";
import { AllianceStation, ArenaState, LoadedMatch, SerialisedMatchGeneration, SerializedMatch } from "ws-schema";
import update from 'immutability-helper';

type FullArena = {
  state?: ArenaState,
  stations?: AllianceStation[],
  match?: LoadedMatch,
};

type FullMatches = {
  quals?: SerializedMatch[],
  playoffs?: SerializedMatch[],
  next?: SerializedMatch
}

type MatchControlState = {
  arena: FullArena,
  matches: FullMatches
}

export default class MatchControl extends React.Component<{ ws: JmsWebsocket }, MatchControlState> {
  readonly state: MatchControlState = { arena: {}, matches: {} };
  handles: string[] = [];

  componentDidMount = () => {
    this.handles = [
      this.props.ws.onMessage<ArenaState>(["Arena", "State", "Current"], 
        msg => this.setState(update(this.state, { arena: { state: { $set: msg } } }))),
      this.props.ws.onMessage<AllianceStation[]>(["Arena", "Alliance", "CurrentStations"], 
        msg => this.setState(update(this.state, { arena: { stations: { $set: msg } } }))),
      this.props.ws.onMessage<LoadedMatch | null>(["Arena", "Match", "Current"], 
        msg => this.setState(update(this.state, { arena: { match: { $set: msg || undefined } } }))),
      this.props.ws.onMessage<SerialisedMatchGeneration>(["Match", "Quals", "Generation"], 
        msg => this.setState(update(this.state, { matches: { quals: { $set: msg.matches } } }))),
      this.props.ws.onMessage<SerialisedMatchGeneration>(["Match", "Playoffs", "Generation"], 
        msg => this.setState(update(this.state, { matches: { playoffs: { $set: msg.matches } } }))),
      this.props.ws.onMessage<SerializedMatch | null>(["Match", "Next"], 
        msg => this.setState(update(this.state, { matches: { next: { $set: msg || undefined } } })))
    ]
  }

  componentWillUnmount = () => this.props.ws.removeHandles(this.handles)

  render() {
    let { ws } = this.props;
    let { arena, matches } = this.state;
    let has_match = !!arena.match;

    return <Container>
      <Row>
        <Col>
          <h3> { arena.match?.match_meta?.name || <i>No Match Loaded</i> } </h3>
        </Col>
        <Col md="auto">
          <Button
            variant="danger"
            onClick={() => ws.send({ Arena: { Match: "Unload" } })}
            disabled={arena.state?.state !== "Idle" || !has_match}
          >
            Unload Match
          </Button>
          &nbsp;
          <Button
            variant="warning"
            onClick={() => ws.send({ Arena: { Match: "LoadTest" } })}
            disabled={arena.state?.state !== "Idle"}
          >
            Load Test Match
          </Button>
        </Col>
      </Row>
      <br />
      <Row >
        <Col>
          <Row>
            <Col>
              <AllianceDash
                colour="blue"
                matchLoaded={ has_match }
                arenaState={arena.state}
                matchScore={arena.match?.score?.blue}
                stations={arena.stations?.filter(x => x.station.alliance === "Blue") || []}
                onStationUpdate={ update => ws.send({ Arena: { Alliance: { UpdateAlliance: update } } }) }
              />
            </Col>
            <Col>
              <AllianceDash
                colour="red"
                matchLoaded={ has_match }
                arenaState={arena.state}
                matchScore={arena.match?.score?.red}
                stations={arena.stations?.filter(x => x.station.alliance === "Red").reverse() || []}  // Red teams go 3-2-1 to order how they're seen from the scoring table
                onStationUpdate={ update => ws.send({ Arena: { Alliance: { UpdateAlliance: update } } }) }
              />
            </Col>
          </Row>
          <br />
          <MatchFlow
            state={arena.state}
            matchLoaded={has_match}
            onSignal={sig => ws.send({ Arena: { State: { Signal: sig } } })}
            onAudienceDisplay={scene => ws.send({ Arena: { AudienceDisplay: { Set: scene } } })}
          />
        </Col>
      </Row>
      <br />
      <Row>
        <Col>
          <MatchScheduleView
            arenaState={arena.state}
            currentMatch={arena.match}
            quals={matches.quals || []}
            playoffs={matches.playoffs || []}
            nextMatch={matches.next}
            onLoadMatch={match_id => ws.send({ Arena: { Match: { Load: match_id } } })}
          />
        </Col>
      </Row>
    </Container>
  }
}