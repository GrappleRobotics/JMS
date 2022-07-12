import { faCheck, faCloudDownloadAlt, faInfoCircle, faSpinner, faSquareCheck, faSquareXmark, faTimes, faTrashCan } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import EditableFormControl from "components/elements/EditableFormControl";
import React from "react";
import { Accordion, Button, Card, Form, FormControl, FormControlProps, Table } from "react-bootstrap";
import { nullIfEmpty } from "support/strings";
import { Importer, ImporterField } from "react-csv-importer";
import { Team, SerialisedMatchGeneration } from "ws-schema";
import { WebsocketContext, WebsocketContextT } from "support/ws-component";
import { EventWizardPageContent } from "./EventWizard";
import confirmBool, { withConfirm } from "components/elements/Confirm";
import { BufferedProps } from "components/elements/BufferedFormControl";
import SimpleTooltip from "components/elements/SimpleTooltip";

// This is a well-known public key I've created. It may be cancelled at any time.
const TBA_AUTH_KEY = "19iOXH0VVxCvYQTlmIRpXyx2xoUQuZoWEPECGitvJcFxEY6itgqDP7A4awVL2CJn";

type ConfigureTeamsState = {
  teams: Team[],
  has_matches: boolean,
  new_team: Partial<Team>,
  fetching: boolean
}

export default class ConfigureTeams extends React.Component {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;
  handles: string[] = [];

  readonly state: ConfigureTeamsState = {
    teams: [],
    has_matches: true,
    new_team: {},
    fetching: false
  };

  componentDidMount = () => {
    this.handles = [
      this.context.listen<Team>(["Event", "Team", "CurrentAll"], msg => this.setState({ teams: msg })),
      this.context.listen<SerialisedMatchGeneration>(["Match", "Quals", "Generation"], msg => this.setState({ has_matches: msg.matches.length > 0 }))
    ]
  }

  componentWillUnmount = () => this.context.unlisten(this.handles);

  updateTeam = (team: Partial<Team>) => {
    if (team.id !== undefined) {
      if (!isNaN(team.id) && team.id > 0) {
        if (team.schedule === undefined)
          team.schedule = true;
        
        this.context.send({
          Event: { Team: { Insert: team as Team } }
        })
        return true;
      } else {
        alert("Team Number must be a number greater than 0");
        return false;
      }
    }
  }

  importCSV = async (rows: { id: string, name?: string, affiliation?: string, location?: string }[]) => {
    rows.forEach(team => {
      let id = nullIfEmpty(team.id);
      if (id !== null) {
        this.updateTeam({
          id: parseInt(id),
          name: nullIfEmpty(team.name),
          affiliation: nullIfEmpty(team.affiliation),
          location: nullIfEmpty(team.location),
          schedule: true
        })
      }
    })
  }

  updateFromTBA = async (override: boolean) => {
    if (override) {
      let result = await confirmBool('Are you sure? This will override all team information.', {
        title: "Force TheBlueAlliance Update",
        okText: "Fetch & Override",
        okVariant: "hazard-red",
        cancelText: "Back to Safety"
      });
      
      if (!result)
        return;
    }

    this.setState({ fetching: true });

    console.log("Starting TBA Update...");
    let promises = this.state.teams.map((t) => (
      fetch("https://www.thebluealliance.com/api/v3/team/frc" + t.id + "?X-TBA-Auth-Key=" + TBA_AUTH_KEY)
        .then(response => response.text())
        .then(JSON.parse)
        .then(msg => {
          let name = msg.nickname;
          let affiliation = msg.school_name;
          let location = [msg.city, msg.state_prov, msg.country].filter(x => x !== null && x !== undefined).join(", ");

          if (name !== "Off-Season Demo Team") {
            this.updateTeam({
              ...t,
              name: nullIfEmpty(override ? (name || t.name) : (t.name || name)),
              affiliation: nullIfEmpty(override ? (affiliation || t.affiliation) : (t.affiliation || affiliation)),
              location: nullIfEmpty(override ? (location || t.location) : (t.location || location))
            });
          }
        })
    ));

    Promise.allSettled(promises)
      .then(() => this.setState({ fetching: false }))
  }

  NewTeamField = (nt_props: { field: keyof Omit<Team, "schedule" | "wpakey">, name: string } & FormControlProps & React.HTMLAttributes<HTMLInputElement>) => {
    const { field, name, type, ...props } = nt_props;
    const mutate = type === "number" ? ((v: string | null) => v === null ? undefined : parseInt(v)) : (v: string | null) => (v === null ? undefined : v);
    
    return <FormControl
      type={type}
      size="sm"
      value={ this.state.new_team[field] || "" }
      placeholder={name}
      { ...props }
      disabled={this.state.has_matches}
      onChange={ e => this.setState({ new_team: { ...this.state.new_team, [field]: mutate(nullIfEmpty(e.target.value)) } }) }
      onKeyDown={ (e: any) => {
        if (e.key === 'Enter' && this.updateTeam(this.state.new_team)) {
          this.setState({ new_team: {} });
        }
      }}
    />
  }

  EditTeamField = (et_props: { field: keyof Omit<Team, "id" | "schedule" | "wpakey">, team: Team } & Omit<BufferedProps, "value"|"onUpdate">) => {
    let { field, team, ...props } = et_props;
    return <EditableFormControl 
      autofocus
      type="text"
      size="sm"
      { ...props }
      value={ team[field] || "" }
      onUpdate={ v => this.updateTeam({ ...team, [field]: nullIfEmpty(v as string) }) }
    />
  }

  render() {
    const { teams, has_matches, fetching } = this.state;
    const tabLabel = teams.length > 0 ? `Configure Teams (${teams.length})` : "Configure Teams";

    const editable = !has_matches;

    return <EventWizardPageContent tabLabel={tabLabel} attention={teams.length < 6}>
      <h4> Configure Teams </h4>
      <p className="text-muted"> 
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; 
        After the Match Schedule is generated, this list can no longer be changed. You need at least 6 teams to generate a schedule.
      </p>

      <Accordion>
        <Card>
          <Accordion.Toggle as={Card.Header} eventKey="0">
            Import Data
          </Accordion.Toggle>
          <Accordion.Collapse eventKey="0">
            <Card.Body>
              {
                !editable ? <p> Teams are LOCKED (matches generated)... </p> :
                  <Importer
                    restartable={true}
                    processChunk={this.importCSV}
                  >
                    <ImporterField name="id" label="Team Number" />
                    <ImporterField name="name" label="Team Name" optional />
                    <ImporterField name="affiliation" label="Team Affiliation (School)" optional />
                    <ImporterField name="location" label="Team Location" optional />
                  </Importer>
              }
            </Card.Body>
          </Accordion.Collapse>
        </Card>
      </Accordion>

      <br />
      <Button onClick={ () => this.updateFromTBA(false) } disabled={fetching}> 
        <FontAwesomeIcon icon={fetching ? faSpinner : faCloudDownloadAlt} spin={fetching} /> &nbsp;
        Update from TBA 
      </Button> &nbsp;
      <Button variant="warning" onClick={ () => this.updateFromTBA(true) } disabled={fetching}>
        <FontAwesomeIcon icon={fetching ? faSpinner : faCloudDownloadAlt} spin={fetching} /> &nbsp;
        Update from TBA (override)
      </Button>
      <br /> <br />
      <Table striped bordered hover size="sm" className="wizard-team-list">
        <thead>
          <tr>
            <th> # </th>
            <th> Name </th>
            <th> Affiliation </th>
            <th> Location </th>
            <th> Scheduled? </th>
            <th> Actions </th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>
              <this.NewTeamField field="id" name="Team #..." type="number" />
            </td>
            <td>
              <this.NewTeamField field="name" name="Team Name..." />
            </td>
            <td>
              <this.NewTeamField field="affiliation" name="Affiliation..." />
            </td>
            <td>
              <this.NewTeamField field="location" name="Location..." />
            </td>
            <td></td>
            <td></td>
          </tr>
          {
            teams.sort(t => t.id)?.map(t => <tr key={t.id} className="wizard-team-row">
              <td data-field="id"> {t.id} </td>
              <td>
                <this.EditTeamField
                  field="name"
                  team={t}
                />
              </td>
              <td>
                <this.EditTeamField
                  field="affiliation"
                  team={t}
                />
              </td>
              <td>
                <this.EditTeamField
                  field="location"
                  team={t}
                />
              </td>
              <td>
                <SimpleTooltip id="sched-tt" tip="Is this team to be included in the Qualification Match Schedule?">
                  <a
                    className={ `text-${t.schedule ? "good" : "bad"}` }
                    onClick={ () => editable ? this.updateTeam({ ...t, schedule: !t.schedule }) : {} }
                  >
                    <FontAwesomeIcon icon={t.schedule ? faSquareCheck : faSquareXmark} />
                  </a>
                </SimpleTooltip>
              </td>
              <td>
                &nbsp;
                {
                  editable ?
                    <a 
                      className="text-danger" 
                      onClick={() => {
                        const del = () => this.context.send({ Event: { Team: { Delete: t.id } } });
                        return withConfirm(del, 
                          `Are you sure you want to delete Team ${t.id}${t.name ? " - " + t.name : ""}?`,
                          { title: "Delete Team", okVariant: "danger", okText: `DELETE ${t.id}` }
                        );
                      }}
                    > 
                      <FontAwesomeIcon icon={faTrashCan} />
                    </a>
                    : <React.Fragment />
                }
              </td>
            </tr>)
          }
        </tbody>
      </Table>
    </EventWizardPageContent>
  }
}