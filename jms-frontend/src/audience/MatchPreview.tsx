import React from "react";
import { Card, Col, Row } from "react-bootstrap";
import { withVal } from "support/util";
import { ALLIANCES } from "support/ws-additional";
import { AllianceStation, LoadedMatch, Alliance, Team, TeamRanking } from "ws-schema";
import AudienceCard from "./AudienceCard";
import BaseAudienceScene from "./BaseAudienceScene";

type AudienceSceneMatchPreviewState = {
  stations: AllianceStation[],
  match?: LoadedMatch,
  teams: Team[],
  rankings: TeamRanking[]
};

export default class AudienceSceneMatchPreview extends BaseAudienceScene<{}, AudienceSceneMatchPreviewState> {
  readonly state: AudienceSceneMatchPreviewState = {
    stations: [],
    teams: [],
    rankings: []
  };

  componentDidMount = () => this.handles = [
    this.listen("Arena/Alliance/CurrentStations", "stations"),
    this.listen("Arena/Match/Current", "match"),
    this.listen("Event/Team/CurrentAll", "teams"),
    this.listen("Event/Ranking/CurrentAll", "rankings"),
  ]
  
  renderAllianceTeams = (colour: Alliance) => {
    const { stations, teams, rankings } = this.state;

    return stations.filter(s => s.station.alliance == colour).map(s => withVal(s.team, () => {
      return <Row className="team">
        <Col className="team-number" sm={3}>
          { s.team }
        </Col>
        <Col className="team-name">
          {
            withVal(teams.find(t => t.id == s.team), t => <React.Fragment>
              { t.name || t.affiliation }
              {
                withVal(t.affiliation, a => <React.Fragment>
                  <span className="affiliation">
                    { t.affiliation }
                  </span>
                </React.Fragment>)
              }
            </React.Fragment>)
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

  show = () => {
    const { match } = this.state;

    if (match == null)
      return <div className="audience-field" />
    else
      return <AudienceCard event_name={this.props.details.event_name} className="audience-preview">
        <Row>
          <Col className="audience-card-title" md="auto">
            { match.match_meta.name }
          </Col>
          <Col className="preview-text">
            coming up
          </Col>
        </Row>

        <Row className="match-teams">
          {
            ALLIANCES.map(colour => <Col>
              <Card data-alliance={colour}>
                {
                  withVal(match.match_meta[`${colour}_alliance`], a => <Card.Header>
                    Alliance { a }
                  </Card.Header>)
                }
                <Card.Body>
                  <div className="teams">
                    { this.renderAllianceTeams(colour) }
                  </div>
                </Card.Body>
              </Card>
            </Col>)
          }
        </Row>
      </AudienceCard>
  }
}