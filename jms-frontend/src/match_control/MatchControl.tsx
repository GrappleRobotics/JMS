import { ResourceRequirementMinimap, ResourceRequirementMinimapAccordion } from 'components/ResourceComponents';
import update from 'immutability-helper';
import React from 'react';
import { Button, Col, Container, Row } from "react-bootstrap";
import { withVal } from 'support/util';
import { WebsocketComponent } from "support/ws-component";
import { SerialisedAllianceStation, ArenaState, LoadedMatch, ResourceRequirementStatus, SerialisedMatchGeneration, SerializedMatch } from "ws-schema";
import AllianceDash from "./Alliance";
import MatchFlow from "./MatchFlow";
import MatchScheduleView from "./MatchScheduleView";

type FullArena = {
  state?: ArenaState,
  stations?: SerialisedAllianceStation[],
  match?: LoadedMatch,
};

type FullMatches = {
  quals?: SerializedMatch[],
  playoffs?: SerializedMatch[],
  next?: SerializedMatch
}

type MatchControlState = {
  arena: FullArena,
  matches: FullMatches,
  resource_status?: ResourceRequirementStatus
}

export default class MatchControl extends WebsocketComponent<{}, MatchControlState> {
  readonly state: MatchControlState = { arena: {}, matches: {} };

  componentDidMount = () => {
    this.handles = [
      // Arena State
      this.listenFn<ArenaState>("Arena/State/Current", 
        msg => this.setState(state => update(state, { arena: { state: { $set: msg } } }))),
      // Alliances
      this.listenFn<SerialisedAllianceStation[]>("Arena/Alliance/CurrentStations", 
        msg => this.setState(state => update(state, { arena: { stations: { $set: msg } } }))),
      // Current Match
      this.listenFn<LoadedMatch | null>("Arena/Match/Current", 
        msg => this.setState(state => update(state, { arena: { match: { $set: msg || undefined } } }))),
      // Other Matches
      this.listenFn<SerialisedMatchGeneration>("Match/Quals/Generation", 
        msg => this.setState(state => update(state, { matches: { quals: { $set: msg.matches } } }))),
      this.listenFn<SerialisedMatchGeneration>("Match/Playoffs/Generation", 
        msg => this.setState(state => update(state, { matches: { playoffs: { $set: msg.matches } } }))),
      this.listenFn<SerializedMatch | null>("Match/Next", 
        msg => this.setState(state => update(state, { matches: { next: { $set: msg || undefined } } }))),
      this.listen("Resource/Requirements/Current", "resource_status")
    ];
  }

  render() {
    const { arena, matches, resource_status } = this.state;
    const has_match = !!arena.match;

    return <Container>
      <Row>
        <Col>
          <h3> { arena.match?.match_meta?.name || <i>No Match Loaded</i> } </h3>
        </Col>
        <Col md="auto">
          <Button
            variant="danger"
            onClick={() => this.send({ Arena: { Match: "Unload" } })}
            disabled={arena.state?.state !== "Idle" || !has_match}
          >
            Unload Match
          </Button>
          &nbsp;
          <Button
            variant="warning"
            onClick={() => this.send({ Arena: { Match: "LoadTest" } })}
            disabled={arena.state?.state !== "Idle"}
          >
            Load Test Match
          </Button>
        </Col>
      </Row>
      <br />
      <Row>
        <Col className="match-control-main">
          <Row>
            <Col>
              <AllianceDash
                colour="blue"
                matchLoaded={ has_match }
                arenaState={arena.state}
                matchScore={arena.match?.score?.blue}
                stations={arena.stations?.filter(x => x.station.alliance === "blue") || []}
                onStationUpdate={ update => this.send({ Arena: { Alliance: { UpdateAlliance: update } } }) }
              />
            </Col>
            <Col>
              <AllianceDash
                colour="red"
                matchLoaded={ has_match }
                arenaState={arena.state}
                matchScore={arena.match?.score?.red}
                stations={arena.stations?.filter(x => x.station.alliance === "red").reverse() || []}  // Red teams go 3-2-1 to order how they're seen from the scoring table
                onStationUpdate={ update => this.send({ Arena: { Alliance: { UpdateAlliance: update } } }) }
              />
            </Col>
          </Row>
          <br />
          <MatchFlow
            state={arena.state}
            matchLoaded={has_match}
            onSignal={sig => this.send({ Arena: { State: { Signal: sig } } })}
            onAudienceDisplay={scene => this.send({ Arena: { AudienceDisplay: { Set: scene } } })}
            resources={ resource_status }
            stations={arena.stations || []}
          />
        </Col>
        {
          withVal(resource_status, status => <Col className="match-control-map" md="auto">
            <Row>
              <ResourceRequirementMinimap status={status} />
            </Row>
          </Col>)
        }
        <Col className="match-control-schedule">
          <MatchScheduleView
            arenaState={arena.state}
            currentMatch={arena.match}
            quals={matches.quals || []}
            playoffs={matches.playoffs || []}
            nextMatch={matches.next}
            onLoadMatch={match_id => this.send({ Arena: { Match: { Load: match_id } } })}
          />
        </Col>
      </Row>
    </Container>
  }
}