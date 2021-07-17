import { faCloudSun, faHourglassHalf, faInfoCircle, faPlus, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BufferedFormControl from "components/elements/BufferedFormControl";
import EditableFormControl from "components/elements/EditableFormControl";
import moment from "moment";
import "moment-duration-format";
import React from "react";
import { Accordion, Button, Card, Col, Form, Row } from "react-bootstrap";
import Datetime from "react-datetime";

const TRANSPORT_FORMAT = "YYYY-MM-D HH:mm:ss.S";
const ELEMENT_FORMAT = "YYYY-MM-D[T]HH:mm";

class ScheduleBlock extends React.Component {
  constructor(props) {
    super(props);
  }

  updateDate = (k, v) => {
    v = moment(v);
    if (moment.isMoment(v)) {
      let update = {
        [k]: v.local().format(TRANSPORT_FORMAT)
      };

      if (k == "start_time") {
        // Also push back the end time
        let new_end = this.props.block.end_time.clone().add(moment.duration(v.diff(this.props.block.start_time)));
        update["end_time"] = new_end.local().format(TRANSPORT_FORMAT);
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
    let { block } = this.props;
    let { duration, start_time, end_time, cycle_time, num_matches, quals } = block;

    return <Accordion defaultActiveKey="0">
      <Card border={ quals ? "primary" : "secondary" }>
        <Accordion.Toggle as={Card.Header} eventKey="0">
          <Row>
            <Col className="my-auto">
              <EditableFormControl
                autofocus
                size="sm"
                value={block.name}
                onUpdate={v => this.props.update("name", v)}
              />

              {
                quals ? <i className="text-muted"> (Quals) </i> : <React.Fragment />
              }
            </Col>
            <Col className="text-center">
              { start_time.format("HH:mm") } - { end_time.format("HH:mm") }
              <br />
              <small className="text-muted">
                { duration.format("h [hours], m [minutes]", { trim: "both" }) }
              </small>
            </Col>
            <Col className="text-right my-auto">
              <span className="mx-2">
                { num_matches } Matches
              </span>
              <span className="text-muted">•</span>
              <a onClick={e => { e.stopPropagation(); this.props.delete() }} className="text-danger mx-2">
                <FontAwesomeIcon icon={faTimes} />
              </a>
            </Col>
          </Row>
        </Accordion.Toggle>
        <Accordion.Collapse eventKey="0">
          <Card.Body>
            <Row>
              <Col>
                <Form.Label>Start Time</Form.Label>

                <BufferedFormControl
                  auto
                  type="datetime-local"
                  value={ start_time.format(ELEMENT_FORMAT) }
                  onUpdate={ v => this.updateDate("start_time", v) }
                />
              </Col>
              <Col>
                <Form.Label>End Time</Form.Label>

                <BufferedFormControl
                  auto
                  type="datetime-local"
                  value={ end_time.format(ELEMENT_FORMAT) }
                  onUpdate={ v => this.updateDate("end_time", v) }
                />
              </Col>
              <Col>
                <Form.Label>Cycle Time</Form.Label>
                <BufferedFormControl
                  auto
                  type="time"
                  value={ cycle_time.format("HH:mm:ss", { trim: false }) }
                  step={1000}
                  onUpdate={ v => this.updateTime("cycle_time", v) }
                />
                <Form.Text className="text-muted">
                  { num_matches } matches
                </Form.Text>
              </Col>
            </Row>
            <Row>
              <Col>
                <Form.Check
                  checked={quals}
                  onChange={ v => this.props.update({ quals: v.target.checked }) }
                  label="Qualifications Matches"
                />
              </Col>
            </Row>
          </Card.Body>
        </Accordion.Collapse>
      </Card>
    </Accordion>
    }
}

export default class ConfigureSchedule extends React.Component {
  static eventKey() { return "configure_day_schedule"; }
  static tabName() { return "Configure Schedule"; }

  static isDisabled(d) {
    return (d.teams?.length || 0) < 6;
  }

  static needsAttention(d) {
    return !!!d.blocks?.length;
  }

  // Map the block data from props into JS types.
  mapBlockProps = (block) => {
    let start_time = moment(block.start_time, TRANSPORT_FORMAT);
    let end_time = moment(block.end_time, TRANSPORT_FORMAT);
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

  render() {
    let total_matches = 0;
    let last = null;
    let blockarr = [];

    this.props.blocks?.forEach((b) => {
      let thisBlock = this.mapBlockProps(b);
      total_matches += thisBlock.num_matches;

      if (last === null || (last.end_time.date() != thisBlock.start_time.date())) {
        blockarr.push(<div className="text-center mt-4 h5">
          <hr />
          { thisBlock.start_time.format("dddd, MMMM Do") }
        </div>);
      } else if (last !== null) {
        let breakLength = moment.duration(thisBlock.start_time.diff(last.end_time));
        blockarr.push(<p className="text-center text-muted">
          <small><i>
            <FontAwesomeIcon icon={faHourglassHalf} /> &nbsp;
            { breakLength.format("h [hours], m [minutes]", { trim: "both" }) }
          </i></small>
        </p>)
      }

      blockarr.push(<div className="my-3">
        <ScheduleBlock block={thisBlock} update={(dict) => this.update(b, dict)} delete={() => this.deleteBlock(b)} />
      </div>);

      last = thisBlock;
    });

    return <div>
      <h4>Configure Schedule</h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the QUALIFICATION schedule is blocked out. Be sure to 
        leave time for breaks, ceremonies, as well as the playoff schedule that will be
        generated later. The playoffs are usually held in the 2nd half of the last day.
        The qualification match schedule will be generated in the next step.
      </p>
      
      <div>
        <Button onClick={this.addBlock}> <FontAwesomeIcon icon={faPlus} /> &nbsp; Add Block </Button>
        <span className="mx-3 float-right">
          <strong>{ total_matches }</strong> matches
          <span className="text-muted mx-2">•</span>
          <strong>{ Math.floor((total_matches * 6) / (this.props.teams?.length || 1)) }</strong> per team
        </span>
      </div>

      <React.Fragment>
        {
          blockarr
        }
      </React.Fragment>
    </div>
  }
}