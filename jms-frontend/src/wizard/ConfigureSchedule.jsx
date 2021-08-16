import { faDownload, faExclamationTriangle, faHourglassHalf, faInfoCircle, faPlus, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BufferedFormControl from "components/elements/BufferedFormControl";
import EditableFormControl from "components/elements/EditableFormControl";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import moment from "moment";
import "moment-duration-format";
import React from "react";
import { Accordion, Button, Card, Col, Form, Row, Modal } from "react-bootstrap";

const ELEMENT_FORMAT = "YYYY-MM-DD[T]HH:mm";

const BLOCK_TYPES = [ "General", "Qualification", "Playoff" ];
const MATCH_BLOCK_TYPES = [ "Qualification", "Playoff" ];

class Load2DayDefault extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      start_date: moment()
    };
  }

  render() {
    let { show, onHide, onSubmit } = this.props;

    return <Modal
      centered
      show={show}
      onHide={onHide}
    >
      <Modal.Header> <h4> Load 2-day Default </h4> </Modal.Header>
      <Modal.Body>
        <p> <FontAwesomeIcon icon={faExclamationTriangle} /> &nbsp; This will override any existing schedule! </p>
        <p> Start Date: </p>
        <BufferedFormControl
          auto
          type="date"
          value={ this.state.start_date.format("YYYY-MM-DD") }
          onUpdate={ v => this.setState({ start_date: moment(v) }) }
        />
      </Modal.Body>
      <Modal.Footer>
        <Button variant="primary" onClick={() => { onSubmit(this.state.start_date); onHide() }}>
          Load
        </Button>
      </Modal.Footer>
    </Modal>
  }
}

class ScheduleBlock extends React.Component {
  updateDate = (k, v) => {
    v = moment(v);
    if (moment.isMoment(v)) {
      let update = {
        [k]: v.local().unix()
      };

      if (k == "start_time") {
        // Also push back the end time
        let new_end = this.props.block.end_time.clone().add(moment.duration(v.diff(this.props.block.start_time)));
        update["end_time"] = new_end.local().unix();
      }

      this.props.update(update);
    }
  }

  updateTime = (k, v) => {
    v = moment.duration(v);
    if (moment.isDuration(v))
      this.props.update({ [k]: v.valueOf() });  // To millis
  }

  render() {
    let { block, disabled } = this.props;
    let { duration, start_time, end_time, cycle_time, num_matches, block_type } = block;

    let is_match_block = MATCH_BLOCK_TYPES.includes(block_type);

    return <Accordion defaultActiveKey={null}>
      <Row>
        <Col md="1" className="text-center text-muted">
          <small>
            { start_time.format("HH:mm") }
            <br />
            { end_time.format("HH:mm") }
          </small>
        </Col>
        <Col md="11">
          <Card border={ is_match_block ? "primary" : "secondary" }>
            <Accordion.Toggle as={Card.Header} eventKey="0">
              <Row>
                <Col md={5}>
                  <EditableFormControl
                    autofocus
                    disabled={disabled}
                    size="sm"
                    value={block.name}
                    onUpdate={v => this.props.update({ name: v })}
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
                <Col md={4} className="text-right">
                  {
                    is_match_block ? 
                      <span>
                        { num_matches } Matches
                        <span className="text-muted mx-2">•</span>
                      </span> : <React.Fragment />
                  }
                  {
                    disabled ? <React.Fragment /> : 
                      <a onClick={e => { e.stopPropagation(); this.props.delete() }} className="text-danger mx-2">
                        <FontAwesomeIcon icon={faTimes} />
                      </a>
                  }
                </Col>
              </Row>
            </Accordion.Toggle>
            <Accordion.Collapse eventKey="0">
              <Card.Body>
                <Row className="mb-3">
                  <Col>
                    <EnumToggleGroup
                      name="block_type"
                      value={block_type}
                      onChange={ v => this.props.update({ block_type: v }) }
                      values={BLOCK_TYPES}
                      outline
                      variant="light"
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
                      onUpdate={ v => this.updateDate("start_time", v) }
                      disabled={disabled}
                    />
                  </Col>
                  <Col>
                    <Form.Label>End Time</Form.Label>

                    <BufferedFormControl
                      auto
                      type="datetime-local"
                      value={ end_time.format(ELEMENT_FORMAT) }
                      onUpdate={ v => this.updateDate("end_time", v) }
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
                          onUpdate={ v => this.updateTime("cycle_time", "00:" + (v || 0) + ":" + cycle_time.seconds()) }
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
                          onUpdate={ v => this.updateTime("cycle_time", "00:" + cycle_time.minutes() + ":" + (v || 0)) }
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
              </Card.Body>
            </Accordion.Collapse>
          </Card>
        </Col>
      </Row>
      
    </Accordion>
    }
}

export default class ConfigureSchedule extends React.Component {
  static eventKey() { return "configure_day_schedule"; }
  static tabName() { return "Configure Schedule"; }

  static needsAttention(d) {
    return !!!d.schedule?.length;
  }

  constructor(props) {
    super(props);

    this.state = {
      show2DayDefault: false
    }
  }

  editable = () => {
    let quals = this.props.matches?.quals;
    if (quals) {
      return !quals.record && !quals.running;
    }
    return true;
  }

  // Map the block data from props into JS types.
  mapBlockProps = (block) => {
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

  addBlock = () => {
    this.props.ws.send("event", "schedule", "new_block");
  }

  update = (block, dict) => {
    this.props.ws.send("event", "schedule", "update_block", {
      ...block,
      ...dict
    });
  }

  deleteBlock = (block) => {
    this.props.ws.send("event", "schedule", "delete_block", block.id);
  }

  loadDefault = v => {
    this.props.ws.send("event", "schedule", "load_default", v.unix());
  }

  render() {
    let total_matches = 0;
    let last = null;
    let blockarr = [];

    this.props.schedule?.forEach((b) => {
      let thisBlock = this.mapBlockProps(b);
      if (thisBlock.block_type == "Qualification")
        total_matches += thisBlock.num_matches;

      if (last === null || (last.end_time.date() != thisBlock.start_time.date())) {
        blockarr.push(<div className="text-center mt-4 h5">
          <hr />
          { thisBlock.start_time.format("dddd, MMMM Do") }
        </div>);
      } else if (last !== null) {
        let breakLength = moment.duration(thisBlock.start_time.diff(last.end_time));
        if (breakLength.asSeconds() > 0) {
          blockarr.push(<p className="text-center text-muted">
            <small><i>
              <FontAwesomeIcon icon={faHourglassHalf} /> &nbsp;
              { breakLength.format("h [hours], m [minutes]", { trim: "both" }) }
            </i></small>
          </p>)
        }
      }

      blockarr.push(<div className="my-3">
        <ScheduleBlock disabled={!this.editable()} block={thisBlock} update={(dict) => this.update(b, dict)} delete={() => this.deleteBlock(b)} />
      </div>);

      last = thisBlock;
    });

    const num_teams = this.props.teams?.filter(t => t.schedule).length || 0;

    let matches_per_team = Math.floor((total_matches * 6) / (num_teams || 1));

    return <div>
      <h4>Configure Schedule</h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the schedule is blocked out. The schedule includes not only qualification matches, but also ceremonies, alliance selection, and 
        playoff matches. To get started, load the 2-day default for a typical 2-day offseason event with a 13 minute gap between matches.
      </p>
      
      <div>
        <Button disabled={!this.editable()} onClick={this.addBlock}> <FontAwesomeIcon icon={faPlus} /> &nbsp; Add Block </Button> &nbsp;
        <Button disabled={!this.editable()} onClick={() => this.setState({ show2DayDefault: true })} variant="info">
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

      <React.Fragment>
        {
          blockarr
        }
      </React.Fragment>

      <Load2DayDefault
        show={this.state.show2DayDefault} 
        onHide={() => this.setState({ show2DayDefault: false })}
        onSubmit={this.loadDefault}
      />
    </div>
  }
}