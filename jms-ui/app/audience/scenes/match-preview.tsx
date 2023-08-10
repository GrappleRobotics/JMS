import { Alliance, AllianceStation, EventDetails, Match, SerialisedLoadedMatch, Team, TeamRanking } from "@/app/ws-schema"
import { useEffect, useState } from "react"
import AudienceCard from "../card";
import React from "react";
import { useWebsocket } from "@/app/support/ws-component";
import { Card, Col, Row } from "react-bootstrap";
import { withVal } from "@/app/support/util";
import { ALLIANCES, ALLIANCES_FLIPPED } from "@/app/support/alliances";

interface MatchPreviewProps {
  eventDetails: EventDetails,
  currentMatch: SerialisedLoadedMatch | null,
  matches: Match[],
  teams: Team[],
  rankings: TeamRanking[],
  stations: AllianceStation[]
}

export default function MatchPreviewSceneInner({ eventDetails, currentMatch, matches, teams, rankings, stations }: MatchPreviewProps) {
  const match = matches.find(m => m.id === currentMatch?.match_id);
  
  const renderAllianceTeams = (colour: Alliance) => {
    return stations.filter(s => s.id.alliance == colour).map(s => withVal(s.team, (t) => {
      const team = teams.find(x => x.number === t);
      return <Row className="team">
        <Col className="team-number" sm={3}>
          { team?.display_number || t }
        </Col>
        <Col className="team-name">
          {
            team && <React.Fragment>
              { team.name || team.affiliation }
              {
                team.affiliation && <React.Fragment>
                  <span className="affiliation">
                    { team.affiliation }
                  </span>
                </React.Fragment>
              }
            </React.Fragment>
          }
        </Col>
        <Col className="team-rank" sm="auto">
          { 
            withVal(rankings.findIndex(r => r.team == s.team) + 1 || undefined, r => `#${r}`)
          }
        </Col>
      </Row>
    }))
  }


  return <AudienceCard event_name={eventDetails.event_name} className="audience-preview">
    <Row>
      <Col className="audience-card-title" md="auto">
        { match?.name || currentMatch?.match_id }
      </Col>
      <Col className="preview-text">
        coming up
      </Col>
    </Row>

    <Row className="match-teams">
      {
        ALLIANCES_FLIPPED.map(colour => <Col>
          <Card data-alliance={colour}>
            {
              withVal(match?.[`${colour}_alliance`], a => <Card.Header>
                Alliance { a }
              </Card.Header>)
            }
            <Card.Body>
              <div className="teams">
                { renderAllianceTeams(colour) }
              </div>
            </Card.Body>
          </Card>
        </Col>)
      }
    </Row>
  </AudienceCard>
}