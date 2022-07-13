import { faDownload, faExclamationTriangle, faHourglassHalf, faInfoCircle, faPlus, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BufferedFormControl from "components/elements/BufferedFormControl";
import { confirmModal } from "components/elements/Confirm";
import EditableFormControl from "components/elements/EditableFormControl";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import moment from "moment";
import "moment-duration-format";
import React from "react";
import { Accordion, Button, Card, Col, Form, Row, Modal } from "react-bootstrap";
import { Combine } from "support/util";
import { WebsocketContext, WebsocketContextT } from "support/ws-component";
import { ScheduleBlock, SerialisedMatchGeneration, Team } from "ws-schema";
import { EventWizardPageContent } from "./EventWizard";

const ELEMENT_FORMAT = "YYYY-MM-DD[T]HH:mm";

type MappedScheduleBlock = Combine<{
  start_time: moment.Moment,
  end_time: moment.Moment,
  cycle_time: moment.Duration,
  duration: moment.Duration,
  num_matches: number
}, ScheduleBlock>

function mapBlock(block: ScheduleBlock): MappedScheduleBlock {
  let start_time = moment.unix(block.start_time);
  let end_time = moment.unix(block.end_time);
  let cycle_time = moment.duration(block.cycle_time, 'milliseconds');
  let duration = moment.duration(end_time.diff(start_time));

  return {
    ...block,
    start_time, end_time, cycle_time,
    duration,
    num_matches: Math.floor(duration.asSeconds() / cycle_time.asSeconds())
  };
}

function unmapBlock(block: MappedScheduleBlock): ScheduleBlock {
  return {
    id: block.id,
    block_type: block.block_type,
    name: block.name,
    start_time: block.start_time.local().unix(),
    end_time: block.end_time.local().unix(),
    cycle_time: block.cycle_time.asMilliseconds()
  };
}

type ScheduleBlockComponentProps = {
  block: MappedScheduleBlock,
  disabled: boolean,
  onUpdate: (new_block: MappedScheduleBlock) => void,
  onDelete: (block_id: number) => void
};

const MATCH_BLOCK_TYPES = [ "Qualification", "Playoff" ];
const BLOCK_TYPES = [ "General", "Qualification", "Playoff" ];

class ScheduleBlockComponent extends React.PureComponent<ScheduleBlockComponentProps> {

  updateDate = (block: MappedScheduleBlock, k: keyof MappedScheduleBlock, v: string | number) => {
    let m = moment(v);
    if (moment.isMoment(m)) {
      let update = {
        ...block, [k]: m
      };

      if (k == "start_time") {
        // Also push back the end time
        update.end_time = block.end_time.clone().add(moment.duration(m.diff(block.start_time)));
      }

      this.props.onUpdate(update);
    }
  }

  updateTime = (block: MappedScheduleBlock, k: keyof MappedScheduleBlock, v: string | number) => {
    let d = moment.duration(v);
    if (moment.isDuration(d))
      this.props.onUpdate({ ...block, [k]: d });
  }

  render() {
    let { block, disabled, onUpdate, onDelete } = this.props;
    let { duration, start_time, end_time, cycle_time, num_matches, block_type } = block;

    let is_match_block = MATCH_BLOCK_TYPES.includes(block_type);

    return <Accordion>
      <Row>
        <Col md="1" className="text-center text-muted">
          <small>
            { start_time.format("HH:mm") }
            <br />
            { end_time.format("HH:mm") }
          </small>
        </Col>
        <Col md="11">
          <Accordion.Item eventKey="0" as={Card} border={ is_match_block ? "primary" : "secondary" }>
            <Accordion.Header className="wizard-schedule-header">
              <Row>
                <Col md={5}>
                  <EditableFormControl
                    autofocus
                    disabled={disabled}
                    value={block.name}
                    onUpdate={v => onUpdate({ ...block, name: String(v) })}
                  />

                  {
                    is_match_block ? <i className="text-muted"> ({block_type}) </i> : <React.Fragment />
                  }
                </Col>
                <Col md={3} className="text-center">
                  <small className="text-muted">
                    { duration.format("h [hours], m [minutes]", { trim: "both" }) }
                  </small>
                </Col>
                <Col md={4} className="text-end">
                  {
                    is_match_block ? 
                      <span>
                        { num_matches } Matches
                        <span className="text-muted mx-2">•</span>
                      </span> : <React.Fragment />
                  }
                  {
                    disabled ? <React.Fragment /> : 
                      <a onClick={e => { e.stopPropagation(); if (block.id != null) onDelete(block.id!) }} className="text-danger mx-2">
                        <FontAwesomeIcon icon={faTimes} />
                      </a>
                  }
                </Col>
              </Row>
            </Accordion.Header>
            <Accordion.Body>
              <Row className="mb-3">
                <Col>
                  <EnumToggleGroup
                    name="block_type"
                    value={block_type}
                    onChange={ (v: any) => onUpdate({ ...block, block_type: v as ScheduleBlock["block_type"] }) }
                    values={BLOCK_TYPES}
                    disabled={disabled}
                  />
                </Col>
              </Row>
              <Row>
                <Col>
                  <Form.Label>Start Time</Form.Label>

                  <BufferedFormControl
                    auto
                    type="datetime-local"
                    value={ start_time.format(ELEMENT_FORMAT) }
                    onUpdate={ v => this.updateDate(block, "start_time", v) }
                    disabled={disabled}
                  />
                </Col>
                <Col>
                  <Form.Label>End Time</Form.Label>

                  <BufferedFormControl
                    auto
                    type="datetime-local"
                    value={ end_time.format(ELEMENT_FORMAT) }
                    onUpdate={ v => this.updateDate(block, "end_time", v) }
                    disabled={disabled}
                  />
                </Col>
                <Col md={3}>
                  <Form.Label>Cycle Time</Form.Label>
                  <Row>
                    <Col>
                      <BufferedFormControl
                        auto
                        disabled={disabled || !is_match_block}
                        type="number"
                        min={0}
                        max={59}
                        value={ cycle_time.minutes() }
                        onUpdate={ v => this.updateTime(block, "cycle_time", "00:" + (v || 0) + ":" + cycle_time.seconds()) }
                      />
                    </Col>
                    <Col className="text-muted">
                      Minutes
                    </Col>
                    <Col>
                      <BufferedFormControl
                        auto
                        disabled={disabled || !is_match_block}
                        type="number"
                        min={0}
                        max={59}
                        value={ cycle_time.seconds() }
                        onUpdate={ v => this.updateTime(block, "cycle_time", "00:" + cycle_time.minutes() + ":" + (v || 0)) }
                      />
                    </Col>
                    <Col className="text-muted">
                      Seconds
                    </Col>
                  </Row>

                  <Form.Text className="text-muted">
                    { num_matches } matches
                  </Form.Text>
                </Col>
              </Row>
            </Accordion.Body>
          </Accordion.Item>
        </Col>
      </Row>
      
    </Accordion>
    }
}

type ConfigureScheduleState = {
  schedule: MappedScheduleBlock[],
  quals_generated: boolean,
  teams: Team[]
};

export default class ConfigureSchedule extends React.Component<{}, ConfigureScheduleState> {
  static contextType = WebsocketContext;
  context!: WebsocketContextT;
  handles: string[] = [];

  readonly state: ConfigureScheduleState = {
    schedule: [],
    quals_generated: false,
    teams: []
  }

  componentDidMount = () => {
    this.handles = [
      this.context.listen<ScheduleBlock[]>([ "Event", "Schedule", "CurrentBlocks" ], msg => {
        this.setState({ schedule: msg.map(mapBlock) })
      }),
      this.context.listen<SerialisedMatchGeneration>([ "Match", "Quals", "Generation" ], msg => this.setState({
        quals_generated: msg.record?.data != null || msg.running
      })),
      this.context.listen<Team[]>([ "Event", "Team", "CurrentAll" ], msg => this.setState({ teams: msg }))
    ]
  }

  componentWillUnmount = () => this.context.unlisten(this.handles);
    
  genListElements = (schedule: MappedScheduleBlock[], editable: boolean) => {
    let total_matches = 0;
    let last: MappedScheduleBlock | undefined = undefined;
    let elements: React.ReactNode[] = [];

    schedule.forEach((block) => {
      if (block.block_type === "Qualification")
        total_matches += block.num_matches;
      
      if (last == null || last.end_time.date() !== block.start_time.date()) {
        // New day - show it in the list
        elements.push(<div key={`${block.id}-new`} className="wizard-schedule-day-split">
          { block.start_time.format("dddd, MMMM Do") }
        </div>)
      } else if (last != null) {
        // Same day as the last one - calculate if there's a break
        const break_time = moment.duration(block.start_time.diff(last.end_time));
        if (break_time.asSeconds() > 0) {
          elements.push(<p key={`${block.id}-break`} className="text-center text-muted">
            <small><i>
              <FontAwesomeIcon icon={faHourglassHalf} /> &nbsp;
              { break_time.format("h [hours], m [minutes]", { trim: "both" }) }
            </i></small>
          </p>)
        }
      }

      elements.push(<div className="my-3">
        <ScheduleBlockComponent
          key={block.id}
          disabled={!editable}
          block={block}
          onUpdate={new_mapped_block => this.context.send({
            Event: { Schedule: { UpdateBlock: unmapBlock(new_mapped_block) } }
          })}
          onDelete={id => this.context.send({
            Event: { Schedule: { DeleteBlock: id } }
          })}
        />
      </div>)

      last = block;
    });

    return { total_matches, elements };
  }

  loadDefault = () => {
    confirmModal("", {
      data: moment(),
      title: "Load 2-day default",
      okText: "Load Defaults",
      renderInner: (start_date: moment.Moment, onUpdate) => <React.Fragment>
        <p> <FontAwesomeIcon icon={faExclamationTriangle} /> &nbsp; This will override any existing schedule! </p>
        <p> Start Date: </p>
        <BufferedFormControl
          auto
          type="date"
          value={start_date.format("YYYY-MM-DD")}
          onUpdate={v => onUpdate(moment(v))}
        />
      </React.Fragment>
    }).then(start_date => {
      this.context.send({
        Event: { Schedule: { LoadDefault: start_date.unix() } }
      })
    })
  }

  render() {
    const { schedule, teams, quals_generated } = this.state;

    const num_teams = teams.filter(t => t.schedule).length;
    const { elements, total_matches } = this.genListElements(schedule, !quals_generated);

    let matches_per_team = Math.floor((total_matches * 6) / (num_teams || 1));

    return <EventWizardPageContent tabLabel="Configure Schedule" attention={schedule.length === 0}>
      <h4>Configure Schedule</h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the schedule is blocked out. The schedule includes not only qualification matches, but also ceremonies, alliance selection, and 
        playoff matches. To get started, load the 2-day default for a typical 2-day offseason event with a 13 minute gap between matches.
      </p>
      
      <div>
        <Button
          disabled={quals_generated}
          onClick={() => this.context.send({ Event: { Schedule: "NewBlock" } })}
        > 
          <FontAwesomeIcon icon={faPlus} /> &nbsp; Add Block 
        </Button> 
        &nbsp;
        <Button disabled={quals_generated} onClick={this.loadDefault} variant="info">
          <FontAwesomeIcon icon={faDownload} /> &nbsp; Load 2-day Default
        </Button>

        <span className="mx-3 float-right">
          <i className="text-muted">Quals</i> &nbsp;
          <strong>{ total_matches }</strong> matches
          <span className="text-muted mx-2">•</span>
          <strong>{ num_teams > 6 ? matches_per_team : "--" }</strong> per team
          <span className="text-muted">
            &nbsp; <i>+ {
              num_teams > 6 ? ( (total_matches * 6) - (matches_per_team * num_teams)) : "--"
            } </i>
          </span>
        </span>
      </div>

      <br />

      { elements }
    </EventWizardPageContent>
  }
}