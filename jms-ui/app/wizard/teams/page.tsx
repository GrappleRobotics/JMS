"use client"

import BufferedFormControl, { BufferedProps } from "@/app/components/BufferedFormControl";
import EditableFormControl from "@/app/components/EditableFormControl";
import { useErrors } from "@/app/support/errors";
import { withPermission } from "@/app/support/permissions"
import { useWebsocket } from "@/app/support/ws-component";
import { Team, TeamUpdate, WebsocketRpcRequest } from "@/app/ws-schema";
import { faCheck, faCloudDownloadAlt, faCross, faInfoCircle, faSpinner, faTimes, faTrash } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React, { useEffect, useState } from "react";
import { Button, Form, FormControlProps, Table } from "react-bootstrap";
import update, { Spec } from "immutability-helper";
import { nullIfEmpty } from "@/app/support/strings";
import { withConfirm } from "@/app/components/Confirm";

// This is a well-known public key I've created. It may be cancelled at any time.
const TBA_AUTH_KEY = "19iOXH0VVxCvYQTlmIRpXyx2xoUQuZoWEPECGitvJcFxEY6itgqDP7A4awVL2CJn";

type NewTeamT = Extract<WebsocketRpcRequest, { path: "team/new_team" }>["data"];

export default withPermission(["ManageTeams"], function EventWizardTeams() {
  const [ teams, setTeams ] = useState<Team[]>([]);
  const [ newTeam, setNewTeam ] = useState<NewTeamT>({
    team_number: 0,
    display_number: "",
    affiliation: null,
    location: null,
    name: null
  });
  const [ fetching, setFetching ] = useState(false);
  const { call, subscribe, unsubscribe } = useWebsocket();
  const { addError } = useErrors();
  
  useEffect(() => {
    let cb = subscribe<"team/teams">("team/teams", setTeams);
    return () => { unsubscribe([cb]) }
  }, []);

  const updateTeam = (team_i: number, updates: TeamUpdate[]) => {
    call<"team/update">("team/update", { team_number: teams[team_i].number, updates })
      .then(t => setTeams(update(teams, { [team_i]: { $set: t } })))
      .catch(addError)
  }

  const updateFromTBA = (force: boolean) => {
    setFetching(true);

    console.log("Starting TBA Update...");
    let promises = teams.map((t, i) => (
      fetch("https://www.thebluealliance.com/api/v3/team/frc" + t.number + "?X-TBA-Auth-Key=" + TBA_AUTH_KEY)
        .then(response => response.text())
        .then(JSON.parse)
        .then(msg => {
          let name = msg.nickname;
          let affiliation = msg.school_name;
          let location = [msg.city, msg.state_prov, msg.country].filter(x => x !== null && x !== undefined).join(", ");

          if (name !== "Off-Season Demo Team") {
            let updates = [];
            if (name != null && (force || t.name == null))
              updates.push({ name });
            if (affiliation != null && (force || t.affiliation == null))
              updates.push({ affiliation });
            if (location != null && (force || t.location == null))
              updates.push({ location });
            
            updateTeam(i, updates);
          }
        })
        .catch(addError)
    ));

    Promise.allSettled(promises)
      .then(() => setFetching(false))
  }

  const NewTeamField = (nt_props: { field: keyof NewTeamT, name: string } & FormControlProps & React.HTMLAttributes<HTMLInputElement>) => {
    const { field, name, type, ...props } = nt_props;
    const mutate = type === "number" ? ((v: string | null) => v === null ? null : parseInt(v)) : (v: string | null) => (v === null ? null : v);
    
    return <BufferedFormControl
      autofocus
      type={type}
      size="sm"
      // @ts-ignore
      value={ newTeam[field] || "" }
      placeholder={name}
      { ...props }
      onUpdate={e => setNewTeam(update(newTeam, { [field]: { $set: mutate(nullIfEmpty(e as string)) } }))}
      onEnter={ (v) => {
        let t = newTeam;
        // @ts-ignore
        t[field] = v;
        if (t.team_number === 0) {
          addError("Team Number can't be 0!");
        } else if (teams.filter(x => x.number === t.team_number).length > 0) {
          addError("This team already exists!");
        } else {
          if (nullIfEmpty(t.display_number) === null) {
            t.display_number = "" + t.team_number;
          }

          call<"team/new_team">("team/new_team", t)
            .then(t => {
              setNewTeam({ team_number: 0, display_number: "", affiliation: null, location: null, name: null });
              setTeams(update(teams, { $push: [t] }))
            })
            .catch(addError)
        }
      }}
    />
  }

  const EditTeamField = (et_props: { field: keyof Omit<Team, "number" | "schedule" | "wpakey">, i: number } & Omit<BufferedProps, "value"|"onUpdate">) => {
    let { field, i, ...props } = et_props;
    return <EditableFormControl 
      autofocus
      type="text"
      { ...props }
      value={ teams[i][field] || "" }
      onUpdate={ v => updateTeam(i, [ { [field]: nullIfEmpty(v as string) } as TeamUpdate ]) }
    />
  }

  return <React.Fragment>
    <h3> Team Management </h3>
    <br />
    <Button variant="blue" onClick={() => updateFromTBA(false)} disabled={fetching}>
      <FontAwesomeIcon icon={fetching ? faSpinner : faCloudDownloadAlt} spin={fetching} /> &nbsp;
      Update from TBA
    </Button> &nbsp;
    <Button variant="red" onClick={() => withConfirm(() => updateFromTBA(true))} disabled={fetching}>
      <FontAwesomeIcon icon={fetching ? faSpinner : faCloudDownloadAlt} spin={fetching} /> &nbsp;
      Update from TBA (Override)
    </Button>
    <br /> <br />
    <Table striped hover size="sm">
      <thead>
        <tr>
          <th>#</th>
          <th>Display #</th>
          <th>Name</th>
          <th>Affiliation</th>
          <th>Location</th>
          <th>Scheduled?</th>
          <th>WPA?</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td> <NewTeamField field="team_number" name="9999" type="number" /> </td>
          <td> <NewTeamField field="display_number" name="9999a" /> </td>
          <td> <NewTeamField field="name" name="Team Name" /> </td>
          <td> <NewTeamField field="affiliation" name="Affiliation..." /> </td>
          <td> <NewTeamField field="location" name="Location..." /> </td>
          <td></td>
          <td></td>
          <td></td>
        </tr>
        {
          teams.sort(t => t.number).map((t, i) => {
            return <tr key={t.number}>
              <td> {t.number} </td>
              <td> <EditTeamField i={i} field="display_number" /> </td>
              <td> <EditTeamField i={i} field="name" /> </td>
              <td> <EditTeamField i={i} field="affiliation" /> </td>
              <td> <EditTeamField i={i} field="location" /> </td>
              <td>
                <Button
                  size="sm"
                  variant={ t.schedule ? "success" : "danger" }
                  onClick={() => updateTeam(i, [ { schedule: !t.schedule } ])}
                >
                  <FontAwesomeIcon icon={t.schedule ? faCheck : faTimes} />
                </Button>
              </td>
              <td className={`text-${t.wpakey ? "success" : "danger"}`}> {t.wpakey != null ? <FontAwesomeIcon icon={faCheck} /> : <FontAwesomeIcon icon={faTimes} />} </td>
              <td>
                <Button
                  variant="danger"
                  size="sm"
                  onClick={() => withConfirm(() => {
                    call<"team/delete">("team/delete", { team_number: t.number })
                      .catch(addError)
                  })}
                >
                  <FontAwesomeIcon icon={faTrash} />
                </Button>
              </td>
            </tr>;
          })
        }
      </tbody>
    </Table>
  </React.Fragment>
});