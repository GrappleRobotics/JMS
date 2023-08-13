"use client"
import "./playoffs.scss";
import { ALLIANCES } from "@/app/support/alliances";
import { withValU } from "@/app/support/util";
import { Alliance, Match, MatchType, PlayoffMode, PlayoffModeType, Team } from "@/app/ws-schema";
import React from "react";
import { Col, Row } from "react-bootstrap";
import _ from "lodash";
import ReactFlow, { Handle, Position } from "reactflow";
import 'reactflow/dist/style.css';

const POS: { [k in PlayoffModeType]: { [k in MatchType]: { [k: number]: { x: number, y: number }[] } }} = {
  "Bracket": {
    "Test": {},
    "Qualification": {},
    "Playoff": {
      1: [
        // Quarters
        { x: 0, y: 0 },
        { x: 0, y: 75 },
        { x: 0, y: 150 },
        { x: 0, y: 225 },
      ],
      2: [
        // Semis
        { x: 300, y: 37.5 },
        { x: 300, y: 187.5 }
      ]
    },
    "Final": {
      1: [
        { x: 600, y: 112.5 }
      ]
    }
  },
  "DoubleBracket": {
    "Test": {},
    "Qualification": {},
    "Playoff": {
      1: [
        { x: 0, y: 0 },
        { x: 0, y: 75 },
        { x: 0, y: 150 },
        { x: 0, y: 225 },
      ],
      2: [
        { x: 250, y: 450 },
        { x: 250, y: 600 },
        { x: 250, y: 37.5 },
        { x: 250, y: 187.5 },
      ],
      3: [
        { x: 500, y: 550 },
        { x: 500, y: 400 },
      ],
      4: [
        { x: 750, y: 112.5 },
        { x: 750, y: 475 },
      ],
      5: [
        { x: 1000, y: 400 }
      ]
    },
    "Final": {
      1: [ { x: 1250, y: 275 } ]
    }
  }
};

const EDGES: { [k in PlayoffModeType]: { src: [MatchType, number, number], dst: [ MatchType, number, number, Alliance] }[] } = {
  "Bracket": [
    { src: ["Playoff", 1, 1], dst: ["Playoff", 2, 1, "red"] },
    { src: ["Playoff", 1, 2], dst: ["Playoff", 2, 1, "blue"] },
    { src: ["Playoff", 1, 3], dst: ["Playoff", 2, 2, "red"] },
    { src: ["Playoff", 1, 4], dst: ["Playoff", 2, 2, "blue"] },

    { src: ["Playoff", 2, 1], dst: ["Final", 1, 1, "red"] },
    { src: ["Playoff", 2, 2], dst: ["Final", 1, 1, "blue"] },
  ],
  "DoubleBracket": [
    // Round 1
    { src: [ "Playoff", 1, 1 ], dst: [ "Playoff", 2, 3, "red" ] },
    { src: [ "Playoff", 1, 2 ], dst: [ "Playoff", 2, 3, "blue" ] },
    { src: [ "Playoff", 1, 3 ], dst: [ "Playoff", 2, 4, "red" ] },
    { src: [ "Playoff", 1, 4 ], dst: [ "Playoff", 2, 4, "blue" ] },
    
    // Round 2
    { src: [ "Playoff", 2, 3 ], dst: [ "Playoff", 4, 1, "red" ] },
    { src: [ "Playoff", 2, 4 ], dst: [ "Playoff", 4, 1, "blue" ] },
    { src: [ "Playoff", 2, 1 ], dst: [ "Playoff", 3, 2, "blue" ] },
    { src: [ "Playoff", 2, 2 ], dst: [ "Playoff", 3, 1, "blue" ] },

    // Round 3
    { src: [ "Playoff", 3, 2 ], dst: [ "Playoff", 4, 2, "red" ] },
    { src: [ "Playoff", 3, 1 ], dst: [ "Playoff", 4, 2, "blue" ] },

    // Round 4
    { src: ["Playoff", 4, 1], dst: [ "Final", 1, 1, "red" ] },
    { src: ["Playoff", 4, 2], dst: [ "Playoff", 5, 1, "blue" ] },

    // Round 5
    { src: ["Playoff", 5, 1], dst: [ "Final", 1, 1, "blue" ] },
  ]
}

export default function PlayoffBracketGraph({ matches, next_match, dark_mode, playoff_mode, teams }: { matches: Match[], next_match?: Match, dark_mode?: boolean, playoff_mode: PlayoffModeType, teams?: Team[] }) {
  const filt_matches = matches.filter(m => m.match_type === "Playoff" || m.match_type === "Final");
  const grouped = Object.values(_.groupBy(filt_matches, m => `${m.match_type}-${m.round}-${m.set_number}`));
  const elements = grouped.sort(ms => ms[0].set_number).flatMap(ms => {
    const ty = ms[0].match_type;
    const round = ms[0].round;
    const set = ms[0].set_number;
    const played = ms.every(m => m.played);
    const ready = ms.every(m => m.ready);

    const is_next = ms.some(m => m.id === next_match?.id);

    return [
      {
        id: `${ty}-${round}-${set}`,
        type: 'set',
        data: {
          round: round,
          set: set,
          matches: ms,
          teams: teams,
          next: is_next,
          played: played,
          ready: ready
        },
        position: {
          x: POS[playoff_mode]?.[ty]?.[round]?.[set - 1]?.x,
          y: POS[playoff_mode]?.[ty]?.[round]?.[set - 1]?.y,
        }
      },
    ]
  });

  const edges = EDGES[playoff_mode]!.map(e => { return { 
    id: `${JSON.stringify(e.src)}->${JSON.stringify(e.dst)}`,
    source: `${e.src[0]}-${e.src[1]}-${e.src[2]}`,
    target: `${e.dst[0]}-${e.dst[1]}-${e.dst[2]}`,
    targetHandle: e.dst[3],
    type: 'smoothstep',
    className: "bracket-edge",
    animated: e.src[0] === next_match?.match_type && e.src[1] === next_match?.round && e.src[2] === next_match?.set_number
  } });

  return <React.Fragment>
    <ReactFlow
      className="playoff-bracket-graph"
      data-dark-mode={dark_mode || false}
      nodeTypes={{
        set: EliminationSet as any
      }}
      nodes={elements}
      edges={edges}
      fitView
      // elements={elements as any}
    />
  </React.Fragment>
}

type EliminationSetProps = {
  data: {
    round: number,
    set: number,
    matches: Match[],
    next: boolean,
    played: boolean,
    ready: boolean,
    teams?: Team[]
  }
};

class EliminationSet extends React.PureComponent<EliminationSetProps> {
  render() {
    const { matches, next, played, ready, teams } = this.props.data;

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
              <Row key={alliance as string} className="bracket-alliance-row" data-alliance={alliance}>
                <Col className="bracket-alliance"> { matches[0][`${alliance}_alliance`] || <React.Fragment>?</React.Fragment> } </Col>
                {
                  matches[0][`${alliance}_teams`].filter(t => t != null).map((t, i) => <Col key={i}>
                    { teams?.find(x => x.number === t)?.display_number || t}
                  </Col>)
                }
                <Col className="spacer" />
              </Row>
            )).reverse()
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