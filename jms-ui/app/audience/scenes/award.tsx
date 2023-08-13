import { Award, AwardRecipient, EventDetails, Team } from "@/app/ws-schema";
import React from "react";
import { Col, Row } from "react-bootstrap";
import AudienceCard from "../card";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faTrophy } from "@fortawesome/free-solid-svg-icons";

interface AwardSceneProps {
  eventDetails: EventDetails,
  award_id: string,
  awards: Award[]
  teams: Team[]
}

export default function AwardScene({ eventDetails, award_id, awards, teams }: AwardSceneProps) {
  const award = awards.find(a => a.id === award_id);

  return award && <AudienceCard event_name={eventDetails.event_name} className="audience-award">
    <Row>
      <Col className="award-title" style={{ fontSize: ( award.name.length > 25 ? "1.25em" : "2em") }}>
        <FontAwesomeIcon icon={faTrophy} className="trophy" /> &nbsp;
        { award.name }
        &nbsp; <FontAwesomeIcon icon={faTrophy} className="trophy" />
      </Col>
    </Row>
    <Row className="recipients">
      <Col className="col-full">
        {
          award.recipients.map((recip, i) => <Row className="recipient" key={i}>
            <AwardRecipientComponent teams={teams} {...recip} />
          </Row>)
        }
      </Col>
    </Row>
  </AudienceCard>
}

class AwardRecipientComponent extends React.PureComponent<AwardRecipient & { teams: Team[] }> {
  render() {
    let { team, awardee, teams } = this.props;

    let theTeam = teams.find(t => (""+t.number) === team || t.display_number === team);

    return <Col>
      { awardee && <div className="awardee" data-has-team={!!team}> { awardee } </div> }
      { team && <div className="team-number" data-has-awardee={!!awardee}> { [team, theTeam?.name].filter(x => !!x).join(" - ") } </div> }
    </Col>
  }
}