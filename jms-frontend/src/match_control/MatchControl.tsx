import AllianceDash from "./Alliance";
import MatchFlow from "./MatchFlow";
import React from "react";
import { Button, Col, Container, Row } from "react-bootstrap";
import MatchScheduleView from "./MatchScheduleView";
import { AllianceStation, ArenaState, LoadedMatch, SerialisedMatchGeneration, SerializedMatch } from "ws-schema";
import update from 'immutability-helper';
import { WebsocketContext, WebsocketContextT } from "support/ws-component";

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

export default class MatchControl extends React.Component<{}, MatchControlState> {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;

  readonly state: MatchControlState = { arena: {}, matches: {} };
  handles: string[] = [];

  componentDidMount = () => {
    this.handles = [
      // Arena State
      this.context.listen<ArenaState>(["Arena", "State", "Current"], 
        msg => this.setState(state => update(state, { arena: { state: { $set: msg } } }))),
      // Alliances
      this.context.listen<AllianceStation[]>(["Arena", "Alliance", "CurrentStations"], 
        msg => this.setState(state => update(state, { arena: { stations: { $set: msg } } }))),
      // Current Match
      this.context.listen<LoadedMatch | null>(["Arena", "Match", "Current"], 
        msg => this.setState(state => update(state, { arena: { match: { $set: msg || undefined } } }))),
      // Other Matches
      this.context.listen<SerialisedMatchGeneration>(["Match", "Quals", "Generation"], 
        msg => this.setState(state => update(state, { matches: { quals: { $set: msg.matches } } }))),
      this.context.listen<SerialisedMatchGeneration>(["Match", "Playoffs", "Generation"], 
        msg => this.setState(state => update(state, { matches: { playoffs: { $set: msg.matches } } }))),
      this.context.listen<SerializedMatch | null>(["Match", "Next"], 
        msg => this.setState(state => update(state, { matches: { next: { $set: msg || undefined } } })))
    ]
  }

  componentWillUnmount = () => this.context.unlisten(this.handles)

  render() {
    const { arena, matches } = this.state;
    const { send } = this.context;
    const has_match = !!arena.match;

    return <Container>
      <Row>
        <Col>
          <h3> { arena.match?.match_meta?.name || <i>No Match Loaded</i> } </h3>
        </Col>
        <Col md="auto">
          <Button
            variant="danger"
            onClick={() => send({ Arena: { Match: "Unload" } })}
            disabled={arena.state?.state !== "Idle" || !has_match}
          >
            Unload Match
          </Button>
          &nbsp;
          <Button
            variant="warning"
            onClick={() => send({ Arena: { Match: "LoadTest" } })}
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
                onStationUpdate={ update => send({ Arena: { Alliance: { UpdateAlliance: update } } }) }
              />
            </Col>
            <Col>
              <AllianceDash
                colour="red"
                matchLoaded={ has_match }
                arenaState={arena.state}
                matchScore={arena.match?.score?.red}
                stations={arena.stations?.filter(x => x.station.alliance === "Red").reverse() || []}  // Red teams go 3-2-1 to order how they're seen from the scoring table
                onStationUpdate={ update => send({ Arena: { Alliance: { UpdateAlliance: update } } }) }
              />
            </Col>
          </Row>
          <br />
          <MatchFlow
            state={arena.state}
            matchLoaded={has_match}
            onSignal={sig => send({ Arena: { State: { Signal: sig } } })}
            onAudienceDisplay={scene => send({ Arena: { AudienceDisplay: { Set: scene } } })}
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
            onLoadMatch={match_id => send({ Arena: { Match: { Load: match_id } } })}
          />
        </Col>
      </Row>
    </Container>
  }
}