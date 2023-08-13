"use client"
import { Alliance, MatchScoreSnapshot, Penalties, ScoreUpdate, ScoreUpdateData, SnapshotScore } from "@/app/ws-schema";
import { Button, Col, Row } from "react-bootstrap";

export function RefereePanelFouls({ score, onUpdate, flipped }: { score: MatchScoreSnapshot, onUpdate: (u: ScoreUpdateData) => void, flipped: boolean }) {
  let foul_cards = [
    <RefereeFoulCard key="bfoul" score={score.blue} alliance="blue" type="fouls" onChange={by => onUpdate({ alliance: "blue", update: { Penalty: { fouls: by } } })} />,
    <RefereeFoulCard key="btech" score={score.blue} alliance="blue" type="tech_fouls" onChange={by => onUpdate({ alliance: "blue", update: { Penalty: { tech_fouls: by } } })} />,
    <RefereeFoulCard key="rfoul" score={score.red} alliance="red" type="tech_fouls" onChange={by => onUpdate({ alliance: "red", update: { Penalty: { tech_fouls: by } } })} />,
    <RefereeFoulCard key="rtech" score={score.red} alliance="red" type="fouls" onChange={by => onUpdate({ alliance: "red", update: { Penalty: { fouls: by } } })} />,
  ];

  return <Row>
    { flipped ? foul_cards.reverse() : foul_cards }
  </Row>
}

export function RefereeFoulCard({ score, alliance, type, onChange }: { score: SnapshotScore, alliance: Alliance, type: keyof Penalties, onChange: (by: number) => void }) {
  const categories: { [k in keyof Penalties]: string } = {
    "fouls": "FOUL",
    "tech_fouls": "TECHNICAL FOUL"
  };

  const category = categories[type];

  return <Col className="penalty-category" data-alliance={alliance}>
    <Row>
      <Col className="penalty-count"> { score.live.penalties[type] } </Col>
    </Row>
    <Row>
      <Col>
        <Button
          className="btn-block btn-penalty"
          data-penalty-type={type}
          variant={`${alliance}`}
          onClick={() => onChange(1)}
        >
          {category}
        </Button>
        <Button
          className="btn-block btn-penalty"
          data-penalty-type={type}
          variant="secondary"
          onClick={() => onChange(-1)}
        >
          SUBTRACT
        </Button>
      </Col>
    </Row>
  </Col>
}