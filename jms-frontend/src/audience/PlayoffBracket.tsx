import { Col, Row } from "react-bootstrap";
import _ from "lodash";
import { MatchGenerationRecordData, SerialisedMatchGeneration, SerializedMatch } from "ws-schema"
import AudienceCard from "./AudienceCard";
import BaseAudienceScene from "./BaseAudienceScene";
import PlayoffBracketGraph, { as_playoffs } from "components/PlayoffBracket";
import React from "react";

type AudienceScenePlayoffBracketState = {
  gen_record?: SerialisedMatchGeneration,
  next?: SerializedMatch
};

export default class AudienceScenePlayoffBracket extends BaseAudienceScene<{}, AudienceScenePlayoffBracketState> {
  readonly state: AudienceScenePlayoffBracketState = { };

  componentDidMount = () => this.handles = [
    this.listen("Match/Playoffs/Generation", "gen_record"),
    this.listen("Match/Next", "next")
  ];

  show = () => {
    const { gen_record, next } = this.state;
    // const gen_record_data = (gen_record?.record?.data as (PlayoffGenRecordData | undefined));
    const data = as_playoffs(gen_record?.record?.data);

    return <AudienceCard event_name={this.props.details.event_name} className="audience-playoff-bracket">
      <Row>
        <Col md={7} className="audience-card-title"> Playoff Bracket </Col>
      </Row>
      <Row className="grow">
        {
          data?.mode === "Bracket"
            ? <PlayoffBracketGraph
                gen_record={gen_record!}
                next={next}
              /> : data?.mode === "RoundRobin"
            ? undefined : undefined
        }
      </Row>
    </AudienceCard>
  }

}
