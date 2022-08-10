import { faCheck, faInfoCircle, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { withConfirm } from "components/elements/Confirm";
import { ResourceRoleLabel } from "components/elements/ResourceComponents";
import SimpleTooltip from "components/elements/SimpleTooltip";
import React from "react";
import { Accordion, Button, Card, Col, ListGroup, Row, Table } from "react-bootstrap";
import { interleave, withVal } from "support/util";
import { ALLIANCES, ALLIANCE_STATIONS } from "support/ws-additional";
import { WebsocketComponent } from "support/ws-component";
import { Alliance, ResourceRequirements, ResourceRequirementStatus, TaggedResource, RefereeID, ScorerID } from "ws-schema";
import { EventWizardPageContent } from "./EventWizard";

type ConfigureResourcesState = {
  requirement_status: ResourceRequirementStatus | null,
  resources: TaggedResource[],
  this_resource?: TaggedResource
}

function referee_quota(id: RefereeID): ResourceRequirements {
  return {
    Quota: {
      template: {
        ready: true,
        role: { RefereePanel: id }
      },
      min: 1,
      max: 1
    }
  };
}

function scorer_quota(id: ScorerID): ResourceRequirements {
  return {
    Quota: {
      template: {
        fta: false,
        ready: true,
        role: { ScorerPanel: id }
      },
      min: 1,
      max: 1
    }
  };
}

const ALL_REFS: ResourceRequirements[] = [
  referee_quota("HeadReferee"),
  referee_quota({ Alliance: [ "blue", "near" ] }),
  referee_quota({ Alliance: [ "blue", "far" ] }),
  referee_quota({ Alliance: [ "red", "near" ] }),
  referee_quota({ Alliance: [ "red", "far" ] }),
];

const ALL_SCORERS: ResourceRequirements[] = [
  scorer_quota({ goals: "AB", height: "high" }),
  scorer_quota({ goals: "AB", height: "low" }),
  scorer_quota({ goals: "CD", height: "high" }),
  scorer_quota({ goals: "CD", height: "low" }),
];

const ALL_ESTOPS: ResourceRequirements[] = ALLIANCES.flatMap(alliance => ALLIANCE_STATIONS.map(stn => (
  { Quota: { template: { role: { TeamEStop: { alliance: alliance, station: stn } } }, min: 1 } }
)));

const ELECTRONICS_OR_ESTOPS: ResourceRequirements = {
  Or: [
    { Quota: { template: { role: "FieldElectronics" }, min: 1 } },
    { And: ALL_ESTOPS }
  ]
}

const DEFAULTS: { [k: string]: ResourceRequirements } = {
  "All Field Resources": {
    And: [
      { Quota: { template: { fta: true, ready: true, role: "Any" }, min: 1 } },
      { Quota: { template: { role: "ScorekeeperPanel" }, min: 1, max: 1 } },
      { Quota: { template: { role: "TimerPanel" }, min: 2 } },
      { Quota: { template: { role: "AudienceDisplay" }, min: 1 } },
      ELECTRONICS_OR_ESTOPS,
      ...ALL_REFS,
      ...ALL_SCORERS
    ]
  }
};

export default class ConfigureResources extends WebsocketComponent<{}, ConfigureResourcesState> {
  readonly state: ConfigureResourcesState = {
    requirement_status: null,
    resources: []
  };

  componentDidMount = () => this.handles = [
    this.listen("Resource/All", "resources"),
    this.listen("Resource/Current", "this_resource"),
    this.listen("Resource/Requirements/Current", "requirement_status")
  ];

  load = (rr: ResourceRequirements | null) => {
    this.send({ Resource: { Requirements: { SetActive: rr } } })
  }

  render() {
    let { requirement_status, resources, this_resource } = this.state;
    return <EventWizardPageContent tabLabel="Resources Allocation" attention={requirement_status == null}>
      <h4> Resource Allocation </h4>
      <p className="text-muted"> 
        <FontAwesomeIcon icon={faInfoCircle} /> &nbsp; 
        Resource Allocation tracks tablets, field electronics, and other resources to determine whether we're ready to run a match.
        Here, you can decide what minimum you require to run a match.
      </p>

      <ActiveResources resources={resources} this_resource={this_resource} />

      <br />
      <div>
        {
          requirement_status == null ? <div>
            <h4> Load Default Allocation </h4>
            <p className="text-muted"> Select from the below default allocations. </p>
            {
              Object.keys(DEFAULTS).map(k => <React.Fragment>
                <Button onClick={() => this.load(DEFAULTS[k])}> { k } </Button>
                &nbsp;
              </React.Fragment>)
            }
          </div> : <Button variant="danger" onClick={() => withConfirm(() => this.load(null), "This will clear the current resource allocation")}> Clear Allocation </Button>
        }
      </div>
      <br />

      {
        withVal(requirement_status, status => <RequirementStatus status={status} resources={resources} />)
      }

    </EventWizardPageContent>
  }
}

class ActiveResources extends React.PureComponent<{ resources: TaggedResource[], this_resource?: TaggedResource }> {
  render() {
    const { resources, this_resource } = this.props;

    return <Accordion>
      <Accordion.Item eventKey="0">
        <Accordion.Header> { resources.length } Active Resource(s) </Accordion.Header>
        <Accordion.Body>
          <Table striped hover size="sm">
            <thead>
              <tr>
                <th> ID </th>
                <th> Role </th>
                <th> Ready? </th>
              </tr>
            </thead>
            <tbody>
              {
                resources.map(r => <tr className="wizard-resource-row" key={r.id} data-fta={r.fta} data-ready={r.ready}>
                  <td> { r.id } { r.id === this_resource?.id ? "(me)" : ""} </td>
                  <td> { r.fta ? <strong>[FTA]&nbsp;</strong> : undefined }  { <ResourceRoleLabel role={r.role} /> } </td>
                  <td className="wizard-resource-ready"> <FontAwesomeIcon icon={ r.ready ? faCheck : faTimes } /> </td>
                </tr>)
              }
            </tbody>
          </Table>
        </Accordion.Body>
      </Accordion.Item>
    </Accordion>
  }
}

class RequirementStatus extends React.PureComponent<{ resources: TaggedResource[], status: ResourceRequirementStatus }> {
  render() {
    const { resources, status } = this.props;
    const { element, satisfied, ready } = status;

    let inner = <React.Fragment />;

    if ("Quota" in element) {
      const quota = element.Quota;
      return <ListGroup.Item className="wizard-resource-status-quota" data-satisfied={satisfied}>
        <Row>
          <Col>
            { quota.template.fta ? "FTA" : <ResourceRoleLabel role={quota.template.role} /> }
          </Col>
          <Col md={2} className="wizard-resource-status-count" data-satisfied={quota.satisfied}>
            <SimpleTooltip tip={ quota.resource_ids.map(r => <p key={r} className="m-0"> { r }</p>) } disabled={quota.resource_ids.length === 0}>
              { quota.resource_ids.length > 0 ? String(quota.resource_ids.length) : "--" }
              &nbsp;&nbsp;
              <span className="text-muted">
                of
                &nbsp;
                { quota.min }{ quota.max ? (quota.max == quota.min ? "" : ` - ${quota.max}`) : "+" }
              </span>
            </SimpleTooltip>
          </Col>
          <Col md={2}>
            {
              satisfied ? <span className={ready ? "" : "text-muted"}>
                <FontAwesomeIcon icon={ready ? faCheck : faTimes} />&nbsp; { ready ? "READY" : "Not Ready" }
              </span>
              : <span className="text-muted">
                <FontAwesomeIcon icon={faTimes} /> &nbsp; Quota
              </span>
            }
          </Col>
        </Row>
      </ListGroup.Item>
    } else {
      let children = ("And" in element) ? element.And : element.Or;

      const or = <ListGroup.Item key="or" className="wizard-resource-status-or">
        <span className="text-muted">OR</span>
      </ListGroup.Item>;

      let list_items = children.map((c, i) => <div key={i} className="wizard-resource-status-item" data-satisfied={c.satisfied}>
        <RequirementStatus resources={resources} status={c} />
      </div>);

      return <Card className="wizard-resource-status" bg={satisfied ? "success" : "dark"}>
        <ListGroup variant="flush">
          {
            "Or" in element ? interleave(list_items, or) : list_items
          }
        </ListGroup>
      </Card>
    }
  }
}