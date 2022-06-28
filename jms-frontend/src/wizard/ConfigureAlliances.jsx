import { faCheck, faExclamationTriangle, faInfoCircle, faUnlock } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BufferedFormControl from "components/elements/BufferedFormControl";
import React from "react";
import { Button, Col, Form, Row, Table, ToggleButton, ToggleButtonGroup } from "react-bootstrap";
import RangeSlider from "react-bootstrap-range-slider";
import { confirm } from "react-bootstrap-confirmation";
import { Menu, MenuItem, Typeahead, TypeaheadMenu } from "react-bootstrap-typeahead";

export default class ConfigureAlliances extends React.Component {
  static eventKey() { return "configure_alliances"; }
  static tabName() { return "Alliance Selection"; }

  static needsAttention(d) {
    return d.alliances?.length === 0 || d.alliances?.some(x => !x.ready);
  }
  
  static isDisabled(d) {
    let qual_matches = d.matches?.quals?.matches;
    if (qual_matches) {
      return qual_matches.length === 0 || !qual_matches.every(x => x.played);
    }
    return true;
  }

  constructor(props) {
    super(props);

    // NOTE: For initial generation ONLY
    this.state = {
      num_teams_per_alliance: 3,
      num_alliances: 2,
      max_num_alliances: null
    }
  }

  componentDidUpdate(prevProps) {
    if (this.props.teams && this.state.max_num_alliances === null) {
      this.updateAllianceCounts(3, 2);
    }
    if (prevProps.teams?.length !== this.props.teams?.length) {
      this.updateAllianceCounts(this.state.num_teams_per_alliance, this.state.num_alliances);
    }
  }

  updateAllianceCounts = (num_teams, num_alliances) => {
    let max_num_alliances = Math.floor(this.props.teams.length / num_teams);
    if (num_teams === 4) {
      let max_with_3 = this.props.teams.length / 3;
      if (max_with_3 > max_num_alliances && this.props.teams.length % 3 > 0) {
        // We can have more alliances with blanks
        max_num_alliances = max_with_3;
      }
    }

    let new_num_alliances = Math.min(num_alliances, max_num_alliances);

    this.setState({
      num_teams_per_alliance: num_teams,
      num_alliances: new_num_alliances,
      max_num_alliances: max_num_alliances
    })
  }

  renderNoAlliances = () => {
    return <div>
      <p className="text-danger">
        <FontAwesomeIcon icon={faExclamationTriangle} /> &nbsp;
        <strong>NOTE: </strong> Your choice for number of alliances will determine what playoff schedules are available to you. 
        For a traditional Quarters - Semis - Finals layout choose 2, 4, or 8 alliances. For other alliance counts, Round Robin or
        Brackets w/ Byes are available.
      </p>

      <h5> Alliance Setup </h5>

      <Row className="my-3">
        <Col>
          <ToggleButtonGroup
            name="teams_per_alliance"
            type="radio"
            value={this.state.num_teams_per_alliance}
            onChange={v => this.updateAllianceCounts(v, this.state.num_alliances)}
          >
            <ToggleButton value={3} variant="outline-primary"> 3 Teams </ToggleButton>
            <ToggleButton value={4} variant="outline-primary"> 4 Teams (3 + Backup) </ToggleButton>
          </ToggleButtonGroup>
        </Col>
        <Col md>
          <RangeSlider 
            value={this.state.num_alliances}
            onChange={e => this.updateAllianceCounts(this.state.num_teams_per_alliance, e.target.value)}
            disabled={this.state.max_num_alliances === 2}
            step={1}
            min={2}
            max={Math.min(this.state.max_num_alliances, 8)}
            tooltipLabel={v => v + " Alliances"}
            tooltip="on"
            tooltipPlacement="top"
          />
        </Col>
      </Row>
      <Row>
        <Col>
          <Button
            variant="success"
            onClick={ () => this.props.ws.send("event", "alliances", "create", this.state.num_alliances) }
          >
            Create Alliances
          </Button>
        </Col>
      </Row>
    </div>
  }

  clearAlliances = async () => {
    let result = await confirm("Are you sure? This will clear all alliances.", {
      title: "Clear Alliances?",
      okButtonStyle: "success"
    });

    if (result)
      this.props.ws.send("event", "alliances", "clear");
  }

  updateAlliancePick = (alliance, idx, value) => {
    let team = value.length > 0 ? parseInt(value[0].id) : null;
    alliance.teams[idx] = team;
    this.props.ws.send("event", "alliances", "update", alliance);
  }

  setAllianceReady = (alliance, ready) => {
    this.props.ws.send("event", "alliances", "update", {
      ...alliance,
      ready
    });
  }

  renderAlliances = () => {
    let chosen_teams = this.props.alliances.flatMap(alliance => alliance.teams).filter(x => !!x);

    let teams_with_strings = this.props.rankings?.map((r, i) => {
      return { id: r.team.toString(), rank: i + 1, disabled: chosen_teams.includes(r.team) }
    });

    let editable = !!!this.props.matches?.playoffs?.record?.data;

    return <div>
      <Row className="my-3">
        <Col>
          <Button
            variant="danger"
            disabled={ this.props.alliances.some(x => x.ready) }
            onClick={this.clearAlliances}
          >
            Reset Alliances
          </Button>
          &nbsp;
          <Button
            variant="primary"
            disabled={ !this.props.alliances.some(x => x.teams[0] == null) }
            onClick={() => this.props.ws.send("event", "alliances", "promote")}
          >
            Promote Captains
          </Button>
        </Col>
      </Row>

      <Row>
        <Col>
          <Table striped bordered hover>
            <thead>
              <tr>
                <th> Alliance </th>
                <th> Captain </th>
                <th> Pick 1 </th>
                <th> Pick 2 </th>
                <th> Backup </th>
                <th> Action </th>
              </tr>
            </thead>
            <tbody>
              {
                this.props.alliances.map(alliance => {
                  let canReady = alliance.teams.slice(0, 3).every(x => x !== null && x !== undefined);
                  return <tr>
                    <td> {alliance.id} </td>
                    {
                      [0, 1, 2, 3].map(i => {
                        let selected_team = teams_with_strings?.find(t => parseInt(t.id) === alliance.teams[i]);

                        return <td>
                          <Typeahead
                            disabled={alliance.ready}
                            id={"alliance-" + alliance.id + "-pick-" + i}
                            labelKey="id"
                            placeholder="----"
                            options={teams_with_strings}
                            highlightOnlyResult={true}
                            selected={ selected_team ? [ selected_team ] : [] }
                            onChange={ t => this.updateAlliancePick(alliance, i, t) }
                            renderMenu={(results, menuProps) => <Menu {...menuProps}>
                              { 
                                results.filter(r => !r.disabled).map((result, idx) => 
                                  <MenuItem option={result} position={idx}> 
                                    <span className="text-muted">{ result.rank }:</span> &nbsp;
                                    { result.id }
                                  </MenuItem>)
                              }
                            </Menu>}
                          />
                        </td>
                      })
                    }
                    <td>
                      {
                        alliance.ready ?
                          <Button disabled={!editable} size="sm" variant="outline-danger" onClick={ () => this.setAllianceReady(alliance, false) }> <FontAwesomeIcon icon={faUnlock} /> </Button> :
                          <Button size="sm" disabled={!canReady} variant={ canReady ? "success" : "secondary" } onClick={ () => this.setAllianceReady(alliance, true) }> <FontAwesomeIcon icon={faCheck} /> </Button>
                      }
                    </td>
                  </tr>
                })
              }
            </tbody>
          </Table>
        </Col>
      </Row>
    </div>
  }

  render() {
    return <div>
      <h4>Alliance Selection</h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the playoff alliances will be selected. Traditionally, this is done in an 8-Alliance, 3-Team playoff bracket with 
        picks chosen in serpentine order (1-8, 8-1, 1-8). These brackets and the teams selected can be chosen in any manner you wish.
      </p>

      {
        this.props.alliances == undefined ? <React.Fragment /> :
          this.props.alliances?.length === 0 ? this.renderNoAlliances() : this.renderAlliances()
      }
    </div>
  }
}