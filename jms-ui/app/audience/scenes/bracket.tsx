import { EventDetails, Match, PlayoffMode, PlayoffModeType, Team } from "@/app/ws-schema";
import AudienceCard from "../card";
import PlayoffBracketGraph from "@/app/components/playoff-graphs/PlayoffBracket";

interface PlayoffBracketSceneProps {
  eventDetails: EventDetails,
  matches: Match[],
  next_match?: Match,
  teams: Team[],
  playoff_mode?: PlayoffModeType
}

export default function PlayoffBracketScene({ eventDetails, matches, next_match, teams, playoff_mode }: PlayoffBracketSceneProps) {
  return <AudienceCard event_name={eventDetails?.event_name} className="audience-playoff-bracket">
    {
      playoff_mode && <PlayoffBracketGraph
        matches={matches}
        next_match={next_match}
        teams={teams}
        playoff_mode={playoff_mode}
      />
    }
    
  </AudienceCard>
}