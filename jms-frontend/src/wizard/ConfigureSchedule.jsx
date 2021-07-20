import { faCloudSun, faDownload, faHourglassHalf, faInfoCircle, faPlus, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import BufferedFormControl from "components/elements/BufferedFormControl";
import EditableFormControl from "components/elements/EditableFormControl";
import moment from "moment";
import "moment-duration-format";
import React from "react";
import { Accordion, Button, Card, Col, Form, Row } from "react-bootstrap";
import { confirm } from "react-bootstrap-confirmation";

const ELEMENT_FORMAT = "YYYY-MM-D[T]HH:mm";

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
    let { block } = this.props;
    let { duration, start_time, end_time, cycle_time, num_matches, quals } = block;

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
          <Card border={ quals ? "primary" : "secondary" }>
            <Accordion.Toggle as={Card.Header} eventKey="0">
              <Row>
                <Col md={5}>
                  <EditableFormControl
                    autofocus
                    size="sm"
                    value={block.name}
                    onUpdate={v => this.props.update({ name: v })}
                  />

                  {
                    quals ? <i className="text-muted"> (Quals) </i> : <React.Fragment />
                  }
                </Col>
                <Col md={3} className="text-center">
                  <small className="text-muted">
                    { duration.format("h [hours], m [minutes]", { trim: "both" }) }
                  </small>
                </Col>
                <Col md={4} className="text-right">
                  {
                    quals ? 
                      <span>
                        { num_matches } Matches
                        <span className="text-muted mx-2">•</span>
                      </span> : <React.Fragment />
                  }
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
                      disabled={!quals}
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
                      label="Qualification Matches"
                    />
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

  static isDisabled(d) {
    return !!d.matches?.quals?.record;
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
      num_matches: block.quals ? Math.floor(duration.asSeconds() / cycle_time.asSeconds()) : 0
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

  loadDefault = async () => {
    if ((this.props.schedule?.length || 0) > 0) {
      let result = await confirm("Are you sure? This will override the current schedule layout. This action cannot be undone.", {
        title: "Load 2-day Schedule Defaults",
        okButtonStyle: "success"
      });

      if (!result)
        return;
    }

    this.props.ws.send("event", "schedule", "load_default");
  }

  render() {
    let total_matches = 0;
    let last = null;
    let blockarr = [];

    this.props.schedule?.forEach((b) => {
      let thisBlock = this.mapBlockProps(b);
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
        <ScheduleBlock block={thisBlock} update={(dict) => this.update(b, dict)} delete={() => this.deleteBlock(b)} />
      </div>);

      last = thisBlock;
    });

    let matches_per_team = Math.floor((total_matches * 6) / (this.props.teams?.length || 1));

    return <div>
      <h4>Configure Schedule</h4>
      <p className="text-muted">
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp;
        In this step, the schedule is blocked out. The schedule includes not only qualification matches, but also ceremonies, alliance selection, and 
        playoff matches. To get started, load the 2-day default for a typical 2-day offseason event with a 13 minute gap between matches.
      </p>
      
      <div>
        <Button onClick={this.addBlock}> <FontAwesomeIcon icon={faPlus} /> &nbsp; Add Block </Button> &nbsp;
        <Button onClick={this.loadDefault} variant="info"> <FontAwesomeIcon icon={faDownload} /> &nbsp; Load 2-day Default </Button>

        <span className="mx-3 float-right">
          <strong>{ total_matches }</strong> matches
          <span className="text-muted mx-2">•</span>
          <strong>{ this.props.teams?.length > 6 ? matches_per_team : "--" }</strong> per team
          <span className="text-muted">
            &nbsp; <i>+ {
              this.props.teams?.length > 6 ? ( (total_matches * 6) - (matches_per_team * this.props.teams?.length)) : "--"
            } </i>
          </span>
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