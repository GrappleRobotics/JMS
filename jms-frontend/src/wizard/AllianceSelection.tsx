import { faCheck, faExclamationTriangle, faInfoCircle, faUnlock } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import confirmBool from "components/elements/Confirm";
import _ from "lodash";
import React from "react";
import { Button, Col, Row, Table, ToggleButton, ToggleButtonGroup } from "react-bootstrap";
import RangeSlider from "react-bootstrap-range-slider";
import { WebsocketContext, WebsocketContextT } from "support/ws-component";
import { PlayoffAlliance, SerialisedMatchGeneration, Team, TeamRanking } from "ws-schema";
import { EventWizardPageContent } from "./EventWizard";
import { Menu, MenuItem, Typeahead } from 'react-bootstrap-typeahead';
import EnumToggleGroup from "components/elements/EnumToggleGroup";

type TypeaheadTeam =  { id: string, rank: number, disabled: boolean };

type AllianceSelectionGeneratorState = {
  n_teams_per_alliance: number,
  n_alliances: number,
  max_n_alliances: number
}

class AllianceSelectionGenerator extends React.Component<{ n_teams: number, gen: (n_alliances: number) => void }, AllianceSelectionGeneratorState> {
  readonly state: AllianceSelectionGeneratorState = {
    n_teams_per_alliance: 3,
    n_alliances: 2,
    max_n_alliances: 2
  };

  componentDidMount = () => this.calculate(3, 2);

  componentDidUpdate(prevProps: { n_teams: number }) {
    if (prevProps.n_teams !== this.props.n_teams) {
      this.calculate(this.state.n_teams_per_alliance, this.state.n_alliances)
    }
  }

  calculate = (n_teams: number, n_alliances: number) => {
    let max_num_alliances = Math.floor(this.props.n_teams / n_teams);

    let new_num_alliances = Math.min(n_alliances, max_num_alliances);

    this.setState({
      n_teams_per_alliance: n_teams,
      n_alliances: new_num_alliances,
      max_n_alliances: max_num_alliances
    })
  }
  
  render() {
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
        <EnumToggleGroup
          name="teams_per_alliance"
          value={this.state.n_teams_per_alliance}
          onChange={v => this.calculate(v, this.state.n_alliances)}
          values={[3, 4]}
          names={["3 Teams (Backup Opt.)", "4 Teams (Backup Req.)"]}
        />
          {/* <ToggleButtonGroup
            name="teams_per_alliance"
            type="radio"
            value={this.state.n_teams_per_alliance}
            onChange={v => this.calculate(v, this.state.n_alliances)}
          >
            <ToggleButton key={3} value={3} variant="outline-primary"> 3 Teams </ToggleButton>
            <ToggleButton key={3} value={4} variant="outline-primary"> 4 Teams (3 + Backup) </ToggleButton>
          </ToggleButtonGroup> */}
        </Col>
        <Col md>
          <RangeSlider 
            value={this.state.n_alliances}
            onChange={e => this.calculate(this.state.n_teams_per_alliance, parseInt(e.target.value))}
            disabled={this.state.max_n_alliances === 2}
            step={1}
            min={2}
            max={Math.min(this.state.max_n_alliances, 8)}
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
            onClick={ () => this.props.gen(this.state.n_alliances) }
          >
            Create Alliances
          </Button>
        </Col>
      </Row>
    </div>
  }
}

type AllianceSelectionState = {
  quals_complete: boolean,
  playoffs_generated: boolean,
  teams: Team[],
  alliances: PlayoffAlliance[],
  rankings: TeamRanking[]
};

export default class AllianceSelection extends React.Component<{}, AllianceSelectionState> {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;
  handles: string[] = [];

  readonly state: AllianceSelectionState = {
    quals_complete: false,
    playoffs_generated: false,
    teams: [], alliances: [], rankings: []
  };

  componentDidMount = () => {
    this.handles = [
      this.context.listen<Team[]>(["Event", "Team", "CurrentAll"], msg => this.setState({ teams: msg })),
      this.context.listen<PlayoffAlliance[]>(["Event", "Alliance", "CurrentAll"], msg => this.setState({ alliances: msg })),
      this.context.listen<TeamRanking[]>(["Event", "Ranking", "CurrentAll"], msg => this.setState({ rankings: msg })),
      this.context.listen<SerialisedMatchGeneration>(["Match", "Quals", "Generation"], msg => this.setState({ quals_complete: msg.matches.length > 0 && _.every(msg.matches, m => m.played) })),
      this.context.listen<SerialisedMatchGeneration>(["Match", "Playoffs", "Generation"], msg => this.setState({ playoffs_generated: msg.record?.data != null })),
    ]
  }

  componentWillUnmount = () => this.context.unlisten(this.handles);

  clearAlliances = async () => {
    let result = await confirmBool("Are you sure? This will clear ALL alliances.", {
      title: "Clear Alliances?",
      okVariant: "danger",
      okText: "Clear Alliances"
    });

    if (result)
      this.context.send({ Event: { Alliance: "Clear" } });
  }

  updateAlliancePick = (alliance: PlayoffAlliance, idx: number, value: TypeaheadTeam[]) => {
    let team = value.length > 0 ? parseInt(value[0].id) : null;
    alliance.teams[idx] = team;
    this.context.send({ Event: { Alliance: { Update: alliance } } });
  }

  renderAlliances = () => {
    const { alliances, teams, rankings, playoffs_generated } = this.state;
    const chosen_teams = alliances.flatMap(alliance => alliance.teams).filter(x => x != null);

    const teams_with_strings: TypeaheadTeam[] = rankings.map((r, i) => {
      return { id: r.team.toString(), rank: i + 1, disabled: chosen_teams.includes(r.team) }
    });

    const editable = !playoffs_generated;

    return <div>
      <Row className="my-3">
        <Col>
          <Button
            variant="danger"
            disabled={ alliances.some(x => x.ready) }
            onClick={this.clearAlliances}
          >
            Reset Alliances
          </Button>
          &nbsp;
          <Button
            variant="primary"
            disabled={ !alliances.some(x => x.teams[0] == null) }
            onClick={() => this.context.send({ Event: { Alliance: "Promote" } })}
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
                <th> Lock </th>
              </tr>
            </thead>
            <tbody>
              {
                alliances.map(alliance => {
                  let canReady = alliance.teams.slice(0, 3).every(x => x != null);
                  return <tr>
                    <td> {alliance.id} </td>
                    {
                      [0, 1, 2, 3].map(i => {
                        let selected_team = teams_with_strings.find(t => parseInt(t.id) === alliance.teams[i]);

                        return <td>
                          <Typeahead
                            disabled={alliance.ready}
                            id={"alliance-" + alliance.id + "-pick-" + i}
                            labelKey="id"
                            placeholder="----"
                            options={teams_with_strings}
                            highlightOnlyResult={true}
                            selected={ selected_team ? [ selected_team ] : [] }
                            onChange={ t => this.updateAlliancePick(alliance, i, t as TypeaheadTeam[]) }
                            renderMenu={(results, menuProps) => <Menu {...menuProps}>
                              { 
                                (results as TypeaheadTeam[]).filter(r => !r.disabled).map((result, idx) => 
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
                      <Button
                        size="sm"
                        disabled={!(editable && canReady)}
                        variant={alliance.ready ? "danger" : canReady ? "good" : "secondary"}
                        onClick={() => this.context.send({
                          Event: { Alliance: { Update: { ...alliance, ready: !alliance.ready } } }
                        })}
                      >
                        <FontAwesomeIcon icon={alliance.ready ? faUnlock : faCheck} />
                      </Button>
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
    const { quals_complete, alliances, teams } = this.state;

    const alliances_ready = alliances.length > 0 && _.every(alliances, a => a.ready);
    const n_teams = teams.filter(t => t.schedule).length;

    return <EventWizardPageContent tabLabel="Alliance Selection" attention={quals_complete && !alliances_ready} disabled={!quals_complete}>
      <h4>Alliance Selection</h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the playoff alliances will be selected. Traditionally, this is done in an 8-Alliance, 3-Team playoff bracket with 
        picks chosen in serpentine order (1-8, 8-1, 1-8). These brackets and the teams selected can be chosen in any manner you wish.
      </p>

      {
        alliances.length > 0 ? 
          this.renderAlliances() : 
          <AllianceSelectionGenerator
            n_teams={n_teams} 
            gen={n => this.context.send({
              Event: { Alliance: { Create: n } }
            })}
          />
      }
    </EventWizardPageContent>
  }
}