import { faTrophy } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { Col, Row } from "react-bootstrap";
import { Award, AwardRecipient, Team } from "ws-schema";
import AudienceCard from "./AudienceCard";
import BaseAudienceScene from "./BaseAudienceScene";

class AwardRecipientComponent extends React.PureComponent<AwardRecipient & { teams: Team[] }> {
  render() {
    let { team, awardee, teams } = this.props;

    let theTeam = teams.find(t => t.id === team);

    return <Col>
      { awardee && <div className="awardee" data-has-team={!!team}> { awardee } </div> }
      { team && <div className="team-number" data-has-awardee={!!awardee}> { [team, theTeam?.name].filter(x => !!x).join(" - ") } </div> }
    </Col>
  }
}

type AudienceSceneAwardState = {
  teams: Team[]
};

export default class AudienceSceneAward extends BaseAudienceScene<Award, AudienceSceneAwardState> {
  readonly state: AudienceSceneAwardState = { teams: [] };
  
  componentDidMount = () => this.handles = [
    this.listen("Event/Team/CurrentAll", "teams")
  ];

  show = (award: Award) => {
    return <AudienceCard event_name={this.props.details.event_name} className="audience-award">
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
              <AwardRecipientComponent teams={this.state.teams} {...recip} />
            </Row>)
          }
        </Col>
      </Row>
    </AudienceCard>
  }
}