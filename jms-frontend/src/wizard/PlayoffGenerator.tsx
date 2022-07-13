import { faCheck, faInfoCircle } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import confirmBool from "components/elements/Confirm";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import _ from "lodash";
import moment from "moment";
import React from "react";
import { Button, Form, Table } from "react-bootstrap";
import { WebsocketContext, WebsocketContextT } from "support/ws-component";
import { MatchGenerationRecordData, PlayoffAlliance, PlayoffMode, SerialisedMatchGeneration } from "ws-schema";
import { EventWizardPageContent } from "./EventWizard";

const PLAYOFF_TYPES: PlayoffMode[]  = [ "Bracket", "RoundRobin" ];
const PLAYOFF_TYPE_NAMES            = [ "Elimination Bracket", "Round Robin" ];

type PlayoffGenRecordData = Extract<MatchGenerationRecordData, { Playoff: any }>;

type PlayoffGeneratorState = {
  alliances: PlayoffAlliance[],
  gen?: SerialisedMatchGeneration,
  playoff_type: PlayoffMode
}

export default class PlayoffGenerator extends React.Component<{}, PlayoffGeneratorState> {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;
  handles: string[] = [];

  readonly state: PlayoffGeneratorState = {
    alliances: [],
    playoff_type: "Bracket"
  };

  componentDidMount = () => {
    this.handles = [
      this.context.listen<PlayoffAlliance[]>(["Event", "Alliance", "CurrentAll"], msg => this.setState({ alliances: msg })),
      this.context.listen<SerialisedMatchGeneration>(["Match", "Playoffs", "Generation"], msg => this.setState({ gen: msg })),
    ]
  }

  componentWillUnmount = () => this.context.unlisten(this.handles);

  clearSchedule = async () => {
    let result = await confirmBool("Are you sure? This will clear the entire Playoffs schedule", {
      title: "Clear Playoffs Schedule?",
      okVariant: "danger",
      okText: "Clear Playoffs"
    });

    if (result)
      this.context.send({ Match: { Playoffs: "Clear" } })
  }

  renderPlayoffs = () => {
    const matches = this.state.gen?.matches || [];
    const max_teams = matches.flatMap(x => [x.blue_teams.length, x.red_teams.length]).reduce((a, b) => Math.max(a, b));
    // const gen_record = (this.state.gen?.record?.data)

    const record_data = (this.state.gen?.record?.data as PlayoffGenRecordData).Playoff;

    return <div>
      <Button
        variant="danger"
        onClick={this.clearSchedule}
        disabled={matches.find(x => x.played) != null}
      >
        Clear Playoff Schedule
      </Button>
      &nbsp;
      <Button
        variant="success"
        onClick={() => this.context.send({ Match: { Playoffs: { Generate: record_data.mode } } })}

      >
        Update
      </Button>
      
      <br /> <br />

      <Table bordered striped size="sm">
        <thead>
          <tr>
            <th> Match </th>
            <th className="schedule-row" data-alliance="blue" colSpan={max_teams}> Blue </th>
            <th className="schedule-row" data-alliance="red" colSpan={max_teams}> Red </th>
          </tr>
        </thead>
        <tbody>
          {
            matches?.map(match => <tr>
              <td> &nbsp; { match.played ? <FontAwesomeIcon icon={faCheck} size="sm" className="text-success" /> : "" } &nbsp; { match.name } </td>
              { Array.from({...match.blue_teams, length: max_teams}).map(t => <td className="schedule-row" data-alliance="blue"> { t } </td>) }
              { Array.from({...match.red_teams, length: max_teams}).map(t =>  <td className="schedule-row" data-alliance="red"> { t } </td>) }
            </tr>)
          }
        </tbody>
      </Table>
    </div>
  }

  playoffTypeSubtitle = () => {
    const n_alliances = this.state.alliances.length;

    switch (this.state.playoff_type) {
      case "Bracket":
        const matches = n_alliances - 1;
        const next_pow = Math.pow(2, Math.ceil(Math.log2(n_alliances)));
        const n_byes = next_pow - n_alliances;
        return <span>
          { matches * 2 } matches ({ matches } sets w/ { n_byes > 0 ? `${n_byes} Byes` : "No Byes" })
        </span>;
      case "RoundRobin":
        const n = n_alliances % 2 == 0 ? n_alliances : (n_alliances + 1);
        // +2 for finals
        return <span>
          { n/2 * (n-1) + 2 } matches ({n_alliances % 2 == 0 ? "No Byes" : "w/ Bye"})
        </span>;
      default:
        return <i> Unknown... </i>;
    }
  }

  renderNoPlayoffs = () => {
    return <div>
      <Form>
        <EnumToggleGroup
          name="playoff_type"
          value={this.state.playoff_type}
          onChange={(t: any) => this.setState({ playoff_type: t as PlayoffMode })}
          names={PLAYOFF_TYPE_NAMES}
          values={PLAYOFF_TYPES}
          disabled={false}
        />
        <br />
        <Form.Text className="text-muted">
          { this.playoffTypeSubtitle() }
        </Form.Text>
      </Form>
      <br />
      <Button
        variant="success"
        // onClick={() => this.props.ws.send("matches", "playoffs", "generate", this.state.playoff_type)}
        onClick={() => this.context.send({ Match: { Playoffs: { Generate: this.state.playoff_type } } })}
      >
        Generate
      </Button>
    </div> 
  }

  render() {
    const has_generated = this.state.gen?.record?.data;
    const prereq = this.state.alliances.length > 0 && _.every(this.state.alliances, a => a.ready);

    return <EventWizardPageContent tabLabel="Generate Playoff Matches" attention={!has_generated} disabled={!prereq}>
      <h4> Generate Playoff Match Schedule </h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the PLAYOFF match schedule is generated. The playoff schedule will update as matches are played.
      </p>

      <div>
        {
          has_generated ? this.renderPlayoffs() : this.renderNoPlayoffs()
        }
      </div>
    </EventWizardPageContent>
  }
}