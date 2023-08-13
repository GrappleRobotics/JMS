import { EventDetails, PlayoffAlliance, Team, TeamRanking } from "@/app/ws-schema";
import AudienceCard from "../card";
import { Col, Row } from "react-bootstrap";

interface AllianceSelectionSceneProps {
  eventDetails: EventDetails,
  alliances: PlayoffAlliance[],
  rankings: TeamRanking[],
  teams: Team[]
}

export default function AllianceSelectionScene({ eventDetails, alliances, rankings, teams }: AllianceSelectionSceneProps) {
  const chosen = alliances.flatMap(a => a.teams).filter(x => !!x);
  const remaining = rankings.filter(r => !chosen.includes(r.team)).map((r, i) => (
    { team: r.team, rank: i + 1 }
  ));

  return <AudienceCard event_name={eventDetails.event_name} className="audience-alliance-selection">
    <Row>
      <Col md={7} className="audience-card-title"> Alliance Selection </Col>
      <Col className="audience-card-title text-end"> Remaining Teams </Col>
    </Row>
    <Row className="grow">
      <Col className="alliance-table" md={7}>
        <Row>
          <Col md={1}> # </Col>
          <Col> Captain </Col>
          <Col> Pick 1 </Col>
          <Col> Pick 2 </Col>
          <Col> Pick 3 </Col>
        </Row>
        {
          alliances.map((alliance, i) => <Row key={i}>
            <Col md={1}> { alliance.number } </Col>
            {
              [0, 1, 2, 3].map(i => <Col key={i}>
                { teams.find(t => t.number === alliance.teams[i])?.display_number || alliance.teams[i] }
              </Col>)
            }
          </Row>)
        }
      </Col>
      <Col className="alliance-remaining">
        <Row className="flex-wrap">
          {
            remaining.map((r, i) => <Col md="auto" key={i}>
              <span className="rank">{ r.rank }</span>: { teams.find(t => t.number === r.team)?.display_number || r.team }
            </Col>)
          }
        </Row>
      </Col>
    </Row>
  </AudienceCard>
}
