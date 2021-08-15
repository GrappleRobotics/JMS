import { faTrophy } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Col, Row } from "react-bootstrap";
import AudienceCard from "./AudienceCard";

class AwardRecipient extends React.PureComponent {
  render() {
    let { team, awardee, all_teams } = this.props;

    let theTeam = all_teams.find(t => t.id === team);

    return <Col>
      { awardee && <div className="awardee" data-has-team={!!team}> { awardee } </div> }
      { team && <div className="team-number" data-has-awardee={!!awardee}> { team } - { theTeam.name } </div> }
    </Col>
  }
}

export default class AudienceAward extends React.PureComponent {
  render() {
    const { event, award } = this.props;

    return <AudienceCard event={event} className="audience-award">
      <Row>
        <Col className="award-title">
          <FontAwesomeIcon icon={faTrophy} className="trophy" /> &nbsp;
          { award.name }
          &nbsp; <FontAwesomeIcon icon={faTrophy} className="trophy" />
        </Col>
      </Row>
      <Row className="recipients">
        <Col className="col-full">
          {
            award.recipients.map(recip => <Row className="recipient">
              <AwardRecipient all_teams={event.teams} {...recip} />
            </Row>)
          }
        </Col>
      </Row>
    </AudienceCard>
  }
}