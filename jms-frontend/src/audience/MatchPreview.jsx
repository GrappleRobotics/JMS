import React from "react";
import { Card, Col, Row } from "react-bootstrap";
import { withVal } from "support/util";
import AudienceCard from "./AudienceCard";

export default class MatchPreview extends React.Component {
  renderAllianceTeams = (colour) => {
    const { event, stations } = this.props;

    return stations.filter(s => s.station.alliance.toLowerCase() == colour).map(s => withVal(s.team, () => {
      return <Row className="team">
        <Col className="team-number" sm={3}>
          { s.team }
        </Col>
        <Col className="team-name">
          {
            withVal(event.teams?.find(t => t.id == s.team), t => <React.Fragment>
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
            withVal(event.rankings?.findIndex(r => r.team == s.team) + 1 || undefined, r => `#${r}`)
          }
        </Col>
      </Row>
    }))
  }

  render() {
    const { match, event } = this.props;

    return <AudienceCard event={event} className="audience-preview">
      <Row>
        <Col className="audience-card-title" md="auto">
          { match.name }
        </Col>
        <Col className="preview-text">
          coming up
        </Col>
      </Row>

      <Row className="match-teams">
        {
          ["red", "blue"].map(colour => <Col>
            <Card data-alliance={colour}>
              {
                withVal(match[`${colour}_alliance`], a => <Card.Header>
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