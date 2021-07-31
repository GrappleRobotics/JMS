import { faCloudDownloadAlt, faInfoCircle, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import EditableFormControl from "components/elements/EditableFormControl";
import React from "react";
import { Accordion, Button, Card, FormControl, Table } from "react-bootstrap";
import { nullIfEmpty } from "support/strings";
import { confirm } from "react-bootstrap-confirmation";
import { Importer, ImporterField } from "react-csv-importer";

// This is a well-known public key I've created. It may be cancelled at any time.
const TBA_AUTH_KEY = "19iOXH0VVxCvYQTlmIRpXyx2xoUQuZoWEPECGitvJcFxEY6itgqDP7A4awVL2CJn";

export default class ConfigureTeams extends React.Component {
  static eventKey() { return "configure_teams"; }
  static tabName(d) {
    let teams = d.teams;
    if (teams === undefined || teams === null || teams.length === 0)
      return "Configure Teams";
    else
      return "Configure Teams (" + teams.length + ")";
  }

  static needsAttention(d) {
    return d.teams?.length < 6;
  }

  // static isDisabled(d) {
  //   return !!d.matches?.quals?.record;
  // }

  constructor(props) {
    super(props);
    this.state = { newTeam: {} };
  }

  // Team names and data can still be edited, but new teams cannot be added and teams cannot
  // be removed.
  editable() {
    let quals = this.props.matches?.quals;
    if (quals) {
      return !quals.record && !quals.running;
    }
    return true;
  }

  updateNewTeam = (k, v) => {
    this.setState({ newTeam: {
      ...this.state.newTeam,
      [k]: v
    }});
  }

  newTeamOnKeyDown = (e) => {
    if (e.key === 'Enter')
      this.trySubmitNewTeam();
  }

  trySubmitNewTeam = () => {
    let nt = this.state.newTeam;
    if (nt.id !== null) {
      let id = parseInt(nt.id);
      if (isNaN(id)) {
        alert("Team number must be a number!");
      } else {
        this.props.ws.send("event", "teams", "insert", {
          id: id,
          name: nt.name,
          affiliation: nt.affiliation,
          location: nt.location
        });
        this.setState({ newTeam: {} });
      }
    }
  }

  importCSV = async (rows) => {
    rows.forEach(team => {
      if (!!nullIfEmpty(team.id)) {
        this.props.ws.send("event", "teams", "insert", {
          id: parseInt(team.id),
          name: nullIfEmpty(team.name),
          affiliation: nullIfEmpty(team.affiliation),
          location: nullIfEmpty(team.location)
        });
      }
    });
  }

  newTeamField = (p) => {
    let {id, name, ...otherProps} = p;
    return <FormControl
      type="text"
      size="sm"
      value={this.state.newTeam[id] || ""}
      onChange={ e => this.updateNewTeam(id, nullIfEmpty(e.target.value)) }
      onKeyDown={this.newTeamOnKeyDown}
      placeholder={name}
      disabled={!this.editable()}
      {...otherProps}
    />
  }

  updateTeam = (team, key, newValue) => {
    let teamDict = {
      ...team,
      [key]: newValue
    };
    this.props.ws.send("event", "teams", "insert", teamDict);
  }

  bufferedField = (p) => {
    let { team, field, ...otherProps } = p;
    return <EditableFormControl
      autofocus
      type="text"
      size="sm"
      value={ team[field] || "" }
      onUpdate={ v => this.updateTeam(team, field, nullIfEmpty(v)) }
      {...otherProps}
    />
  }

  delete = (t) => {
    this.props.ws.send("event", "teams", "delete", t.id);
  }

  updateTBA = async (override) => {
    if (override) {
      let result = await confirm('Are you sure? This will override all team information.', {
        title: "Force TBA Update",
        okButtonStyle: "success"
      });
      
      if (!result)
        return;
    }

    console.log("Starting TBA Update...");
    this.props.teams.forEach((t) => {
      fetch("https://www.thebluealliance.com/api/v3/team/frc" + t.id + "?X-TBA-Auth-Key=" + TBA_AUTH_KEY)
        .then(response => response.text())
        .then(JSON.parse)
        .then(msg => {
          let name = msg.nickname;
          let affiliation = msg.school_name;
          let location = [msg.city, msg.state_prov, msg.country].filter(x => x !== null && x !== undefined).join(", ");

          if (name !== "Off-Season Demo Team") {
            let nt = {
              ...t,
              name: nullIfEmpty(override ? (name || t.name) : (t.name || name)),
              affiliation: nullIfEmpty(override ? (affiliation || t.affiliation) : (t.affiliation || affiliation)),
              location: nullIfEmpty(override ? (location || t.location) : (t.location || location))
            };

            this.props.ws.send("event", "teams", "insert", nt);
          }
        });
    });
  }

  render() {
    return <div>
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
                !this.editable() ? <p> Teams are LOCKED... </p> :
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
      <Button onClick={ () => this.updateTBA(false) }> 
        <FontAwesomeIcon icon={faCloudDownloadAlt} /> &nbsp;
        Update from TBA 
      </Button> &nbsp;
      <Button variant="warning" onClick={ () => this.updateTBA(true) }>
        <FontAwesomeIcon icon={faCloudDownloadAlt} /> &nbsp;
        Update from TBA (override)
      </Button>
      <br /> <br />
      <Table striped bordered hover size="sm">
        <thead>
          <tr>
            <th> # </th>
            <th> Name </th>
            <th> Affiliation </th>
            <th> Location </th>
            <th> Actions </th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>
              <this.newTeamField id="id" name="Team #..." type="number" />
            </td>
            <td>
              <this.newTeamField id="name" name="Team Name..." />
            </td>
            <td>
              <this.newTeamField id="affiliation" name="Affiliation..." />
            </td>
            <td>
              <this.newTeamField id="location" name="Location..." />
            </td>
            <td></td>
          </tr>
          {
            this.props.teams?.sort(a => a.id)?.map(t => <tr>
              <td className="text-right pr-2 w-15"> {t.id} </td>
              <td>
                <this.bufferedField
                  team={t}
                  field="name"
                />
              </td>
              <td>
                <this.bufferedField
                  team={t}
                  field="affiliation"
                />
              </td>
              <td>
                <this.bufferedField
                  team={t}
                  field="location"
                />
              </td>
              <td>
                {
                  this.editable() ?
                    <a className="text-danger" onClick={() => this.delete(t)}> <FontAwesomeIcon icon={faTimes} /> </a>
                    : <React.Fragment />
                }
              </td>
            </tr>)
          }
        </tbody>
      </Table>
    </div>
  }
}