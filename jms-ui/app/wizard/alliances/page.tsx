"use client"

import { withConfirm } from "@/app/components/Confirm"
import { useErrors } from "@/app/support/errors"
import { withPermission } from "@/app/support/permissions"
import { useWebsocket } from "@/app/support/ws-component"
import { Match, PlayoffAlliance, Team, TeamRanking } from "@/app/ws-schema"
import React, { useEffect, useState } from "react"
import { Alert, Button, Table } from "react-bootstrap"
import { Typeahead } from "react-bootstrap-typeahead"
import update from "immutability-helper";

export default withPermission(["ManageAlliances"], function EventWizardAlliances() {
  const [ alliances, setAlliances ] = useState<PlayoffAlliance[]>([]);
  const [ matches, setMatches ] = useState<Match[]>([]);
  const [ teams, setTeams ] = useState<Team[]>([]);
  const [ rankings, setRankings ] = useState<TeamRanking[]>([]);

  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    let cbs = [
      subscribe<"alliances/alliances">("alliances/alliances", setAlliances),
      subscribe<"matches/matches">("matches/matches", setMatches),
      subscribe<"team/teams">("team/teams", setTeams),
      subscribe<"scoring/rankings">("scoring/rankings", setRankings)
    ];
    return () => unsubscribe(cbs);
  }, [])

  const has_quals = matches.filter(x => x.match_type === "Qualification").length > 0;
  const quals_finished = matches.filter(x => x.match_type === "Qualification").every(x => x.played);
  const has_playoffs = matches.filter(x => x.match_type === "Playoff").length > 0;

  const disabled = has_playoffs;
  const has_alliances = alliances.length > 0;

  const not_chosen = rankings?.filter(t => alliances.filter(a => a.teams.includes(t.team)).length == 0 );

  return <React.Fragment>
    <h3> Alliance Selections </h3>
    { !has_quals && <Alert variant="warning"> There are no Qualification Matches in the schedule. Are you sure you want to edit alliances? </Alert> }
    { has_quals && !quals_finished && <Alert variant="warning"> Qualification Matches are still being played! Are you sure you want to edit alliances? </Alert> }
    { disabled && <Alert variant="info"> Alliances can't be edited after Playoffs have begun! </Alert> }

    {
      has_alliances && <Button variant="success" disabled={disabled} onClick={() => call<"alliances/promote">("alliances/promote", null).then(setAlliances).catch(addError)}>
        Promote Captains
      </Button>
    } &nbsp;
    {
      has_alliances ? <Button variant="danger" disabled={disabled} onClick={() => withConfirm(() => call<"alliances/delete_all">("alliances/delete_all", null).then(() => setAlliances([])).catch(addError))}>
        Delete Alliances
      </Button>
      : <Button variant="success" disabled={disabled} onClick={() => call<"alliances/create">("alliances/create", null).then(setAlliances).catch(addError)}>
        Create Alliances
      </Button>
    }
    <br /> <br />
    <Table striped hover bordered>
      <thead>
        <tr>
          <th>#</th>
          <th>Teams</th>
        </tr>
      </thead>
      <tbody>
        {
          teams.length > 0 && alliances.map((alliance, i) => <tr>
            <td> { alliance.number } </td>
            <td>
              <Typeahead
                id={`alliance-${alliance.number}`}
                multiple
                disabled={disabled}
                options={not_chosen.map(x => teams.find(t => t.number === x.team)!.display_number)}
                selected={alliance.teams.map(x => teams.find(t => t.number === x)!.display_number)}
                onChange={display_teams => {
                  let ts = display_teams.map(dt => teams.find(t => t.display_number === dt)!.number);
                  call<"alliances/set_teams">("alliances/set_teams", { number: alliance.number, teams: ts })
                    .then(a => setAlliances(update(alliances, { [i]: { $set: a } })))
                    .catch(addError)
                }}
              />
            </td>
          </tr>)
        }
      </tbody>
    </Table>
  </React.Fragment>
})