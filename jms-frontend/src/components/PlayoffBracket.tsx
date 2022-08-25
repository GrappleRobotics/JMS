import _ from "lodash";
import React from "react";
import { Col, Row } from "react-bootstrap";
import ReactFlow, { Handle, Position } from "react-flow-renderer";
import { withValU } from "support/util";
import { ALLIANCES } from "support/ws-additional";
import { MatchGenerationRecordData, MatchSubtype, SerialisedMatchGeneration, SerializedMatch } from "ws-schema";

export type PlayoffGenRecordData = Extract<MatchGenerationRecordData, { Playoff: any }>;
export function is_playoff(data?: MatchGenerationRecordData | null): data is PlayoffGenRecordData {
  // return (gen_record?.record?.data as (PlayoffGenRecordData | undefined))
  return data != null && "Playoff" in data;
}

export function as_playoffs(data?: MatchGenerationRecordData | null): PlayoffGenRecordData["Playoff"] | undefined {
  if (is_playoff(data))
    return data.Playoff;
  return undefined;
}

export type PlayoffBracketGraphProps = {
  gen_record: SerialisedMatchGeneration,
  next?: SerializedMatch,
  dark_mode?: boolean
};

const X_POS: { [k in MatchSubtype]: number } = {
  "Quarterfinal": 0,
  "Semifinal": 300,
  "Final": 600
};

const Y_POS: { [k in MatchSubtype]: [number, number] } = {
  "Quarterfinal": [ 0, 75 ],
  "Semifinal": [ 37.5, 150 ],
  "Final": [ 112.5, 200 ]
};

export default class PlayoffBracketGraph extends React.PureComponent<PlayoffBracketGraphProps> {
  gen_id = (subtype: MatchSubtype, set: number) => `${subtype}-${set}`;

  render() {
    const { gen_record, next, dark_mode } = this.props;
    const matches = gen_record.matches;

    const grouped = Object.values(_.groupBy(matches, m => `${m.match_subtype}${m.set_number}`));
    const elements = grouped.sort(ms => ms[0].set_number).flatMap(ms => {
      const set = ms[0].set_number;
      const st = ms[0].match_subtype!;
      const played = ms.every(m => m.played);
      const ready = ms.every(m => m.ready);

      const is_next = ms.some(m => m.id === next?.id);

      const next_level = {
        "Quarterfinal": "Semifinal" as MatchSubtype,
        "Semifinal": "Final" as MatchSubtype,
        "Final": undefined
      }[st];

      const edges = next_level ? [{
        id: `${this.gen_id(st, set)}-edge`,
        source: this.gen_id(st, set),
        target: this.gen_id(next_level, Math.floor((set - 1) / 2) + 1),
        targetHandle: (set - 1) % 2 === 0 ? "red" : "blue",
        animated: is_next,
        type: 'smoothstep',
        className: `bracket-edge ${played ? "bracket-edge-played" : ""} ${is_next ? "bracket-edge-next" : ""}`,
        // style: {
        //   strokeWidth: 2,
        //   stroke: played ? "var(--bs-dark)" : is_next ? "purple" : undefined,
        //   opacity: played ? 0.8 : 1
        // }
      }] : [];

      return [
        {
          id: this.gen_id(st, set),
          type: 'set',
          data: {
            subtype: st,
            set: set,
            matches: matches.filter(m => m.match_subtype === st && m.set_number === set),
            next: is_next,
            played: played,
            ready: ready
          },
          position: {
            x: X_POS[st],
            y: Y_POS[st][0] + Y_POS[st][1] * (set - 1)
          }
        },
        ...edges
      ]
    });

    return <React.Fragment>
      <ReactFlow
        className="playoff-bracket-graph"
        data-dark-mode={dark_mode || false}
        nodeTypes={{
          set: EliminationSet as any
        }}
        elements={elements as any}
        onLoad={i => i.fitView()}
      />
    </React.Fragment>
  }
}

type EliminationSetProps = {
  data: {
    subtype: MatchSubtype,
    set: number,
    matches: SerializedMatch[],
    next: boolean,
    played: boolean,
    ready: boolean
  }
};

class EliminationSet extends React.PureComponent<EliminationSetProps> {
  render() {
    const { matches, next, played, ready } = this.props.data;

    if (matches.length === 0)
      return <React.Fragment />;

    const next_match = matches.find(m => !m.played);

    return <div className="bracket-set" data-next={next} data-played={played} data-ready={ready} data-has-next={next_match != null}>
      <Handle
        id="red"
        type="target"
        position={Position.Left}
        style={{ top: "12.5px", left: 0 }}
      />

      <Handle
        id="blue"
        type="target"
        position={Position.Left}
        style={{ top: "37.5px", left: 0 }}
      />


      <Row className="grow">
        <Col>
          {
            ALLIANCES.map(alliance => (
              <Row className="bracket-alliance-row" data-alliance={alliance} data-winner={played && matches.filter(m => m.winner === alliance).length >= 2}>
                <Col className="bracket-alliance"> { matches[0][`${alliance}_alliance`] } </Col>
                {
                  matches[0][`${alliance}_teams`].filter(t => t != null).map(t => <Col>
                    {t}
                  </Col>)
                }
                <Col className="spacer" />
              </Row>
            ))
          }
        </Col>
      </Row>

      {
        withValU(next_match, m => <Row className="match-name">
          <Col> <strong>{ m.name }</strong> <i>{ next ? " (up next)" : "" }</i> </Col>
        </Row>)
      }
      
      <Handle
        type="source"
        position={Position.Right}
        style={{ top: "25px", right: 0 }}
      />
    </div>
  }
}