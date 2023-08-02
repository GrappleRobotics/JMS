"use client";
import "../../match_schedule.scss";
import BufferedFormControl from "@/app/components/BufferedFormControl";
import { confirmModal, withConfirm } from "@/app/components/Confirm";
import { withPermission } from "@/app/support/permissions";
import { useWebsocket } from "@/app/support/ws-component";
import { Match, QualsMatchGeneratorParams } from "@/app/ws-schema";
import React, { useEffect, useState } from "react";
import { Button, InputGroup, Table } from "react-bootstrap";
import update from "immutability-helper";
import JmsWebsocket from "@/app/support/ws";
import { useErrors } from "@/app/support/errors";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faSpinner, faTrash } from "@fortawesome/free-solid-svg-icons";
import { ALLIANCE_STATIONS } from "@/app/support/alliances";
import { capitalise } from "@/app/support/strings";
import moment from "moment";

export default withPermission(["ManageSchedule"], function EventWizardQuals() {
  const [ matches, setMatches ] = useState<Match[]>();
  const [ generationInProgress, setGenerationInProgress ] = useState<boolean>(false);
  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();

  useEffect(() => {
    const cb = [
      subscribe<"matches/matches">("matches/matches", m => setMatches(m.filter(x => x.match_type === "Qualification"))),
      subscribe<"matches/generator_busy">("matches/generator_busy", setGenerationInProgress),
    ];
    return () => unsubscribe(cb);
  }, []);

  return <React.Fragment>
    <h3> Generate Qualification Schedule </h3>
    <Button onClick={() => genQualsModal(call, addError)} disabled={generationInProgress}>
      Generate Qualification Schedule
      { generationInProgress && <FontAwesomeIcon className="ml-2" icon={faSpinner} spin /> }
    </Button> &nbsp;
    <Button
      onClick={() => withConfirm(() => {
        matches?.filter(m => !m.played).forEach(m => call<"matches/delete">("matches/delete", { match_id: m.id }).catch(addError))
      })}
      variant="danger"
    >
      Delete Unplayed
    </Button>
    <br /><br />
    <Table striped bordered size="sm">
      <thead>
        <tr className="schedule-row">
          <th> Time </th>
          <th> Match </th>
          {
            ALLIANCE_STATIONS.map(stn => <th data-alliance={stn[0]}> { capitalise(stn[0] as string) } { stn[1] } </th>)
          }
          <th> Actions </th>
        </tr>
      </thead>
      <tbody>
        {
          matches?.sort((a, b) => moment(a.start_time).unix() - moment(b.start_time).unix())?.map(match => <tr>
            <td> { moment(match.start_time).format("ddd HH:mm:ss") } </td>
            <td> { match.name } </td>
            {
              match.blue_teams.map(t => <td data-alliance="blue">{ t }</td>)
            }
            {
              match.red_teams.map(t => <td data-alliance="red">{ t }</td>)
            }
            <td>
              <Button variant="danger" size="sm" disabled={match.played} onClick={() => withConfirm(() => call<"matches/delete">("matches/delete", { match_id: match.id }).catch(addError))}>
                <FontAwesomeIcon icon={faTrash} />
              </Button>
            </td>
          </tr>)
        }
      </tbody>
    </Table>
  </React.Fragment>
});

async function genQualsModal(call: JmsWebsocket["call"], addError: (e: string) => void) {
  let params = await confirmModal("", {
    title: "Generate Qualification Schedule",
    okText: "Generate",
    data: {
      station_anneal_steps: 50_000,
      team_anneal_steps: 100_000,
    } as QualsMatchGeneratorParams,
    renderInner: (data, onUpdate) => <React.Fragment>
      <InputGroup>
        <InputGroup.Text>Team Anneal Steps</InputGroup.Text>
        <BufferedFormControl
          auto
          value={data.team_anneal_steps}
          onUpdate={v => onUpdate(update(data, { team_anneal_steps: { $set: Math.max(1000, v as number) } }))}
        />
      </InputGroup>
      <InputGroup>
        <InputGroup.Text>Station Anneal Steps</InputGroup.Text>
        <BufferedFormControl
          auto
          value={data.station_anneal_steps}
          onUpdate={v => onUpdate(update(data, { station_anneal_steps: { $set: Math.max(1000, v as number) } }))}
        />
      </InputGroup>
    </React.Fragment>
  });

  call<"matches/gen_quals">("matches/gen_quals", { params })
    .catch(addError);
}