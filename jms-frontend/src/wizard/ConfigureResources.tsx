import { faAdd, faCheck, faCrown, faInfoCircle, faPencil, faTimes, faTrash } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { confirmModal, withConfirm } from "components/elements/Confirm";
import EnumToggleGroup from "components/elements/EnumToggleGroup";
import { ResourceRequirementMinimap, ResourceRequirementMinimapAccordion, ResourceRoleLabel } from "components/ResourceComponents";
import SimpleTooltip from "components/elements/SimpleTooltip";
import BufferedFormControl from "components/elements/BufferedFormControl";
import update, { Spec } from "immutability-helper";
import React from "react";
import { Accordion, Button, Card, Col, Form, ListGroup, Row, Table } from "react-bootstrap";
import { interleave, VariantKeys, withVal, withValU } from "support/util";
import { ALLIANCES, ALLIANCE_STATIONS, ROLES } from "support/ws-additional";
import { WebsocketComponent } from "support/ws-component";
import { ResourceRequirements, ResourceRequirementStatus, TaggedResource, RefereeID, ScorerID, MappedResourceQuota, ResourceQuota, ResourceRole } from "ws-schema";
import { EventWizardPageContent } from "./EventWizard";
import { nullIfEmpty } from "support/strings";

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

const DEFAULT_QUOTA: ResourceRequirements = {
  Quota: {
    template: {
      fta: false,
      ready: false,
      role: "Any"
    },
    min: 1
  }
};

const NEW_DEFAULTS: { Quota: ResourceQuota, And: ResourceRequirements[], Or: ResourceRequirements[] } = {
  ...DEFAULT_QUOTA,
  And: [ DEFAULT_QUOTA ],
  Or: [ DEFAULT_QUOTA ]
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
    return <EventWizardPageContent tabLabel="Resource Allocation" attention={requirement_status == null}>
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
        withVal(requirement_status, status => <React.Fragment>
          <ResourceRequirementMinimapAccordion defaultOpen status={status} />
          <br />
          <RequirementStatusComponent status={status} onUpdate={this.load} />
        </React.Fragment>)
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

type RequirementStatusComponentProps = {
  status: ResourceRequirementStatus,
  onUpdate?: (data: ResourceRequirements | null) => void
};

class RequirementStatusComponent extends React.PureComponent<RequirementStatusComponentProps> {
  render() {
    const { status, onUpdate } = this.props;
    const { element } = status;

    if ("Quota" in element) {
      return <ResourceQuotaComponent
        quota={element.Quota}
        status={status}
        onUpdate={withValU(onUpdate, onUp => up => onUp(up ? { Quota: up } : null))}
      />
    } else if ("And" in element) {
      return <ResourceConjunctiveComponent operator="And" children={element.And} status={status} onUpdate={onUpdate} />
    } else if ("Or" in element) {
      return <ResourceConjunctiveComponent operator="Or" children={element.Or} status={status} onUpdate={onUpdate} />
    }
  }
}

type ResourceConjunctiveComponentProps = {
  children: ResourceRequirementStatus[],
  operator: "And" | "Or",
} & RequirementStatusComponentProps;

class ResourceConjunctiveComponent extends React.PureComponent<ResourceConjunctiveComponentProps> {
  newEntry = (onUpdate: NonNullable<ResourceConjunctiveComponentProps["onUpdate"]>) => {
    confirmModal("", {
      data: { Quota: NEW_DEFAULTS.Quota },
      title: "New Resource Requirement",
      okText: "Add",
      renderInner: (data: ResourceRequirements, onUpdate) => (
        <EditResourceRequirement value={data} onUpdate={s => onUpdate(update(data, s))} />
      )
    }).then(data => {
      onUpdate(update(this.props.status.original, { [this.props.operator]: { $push: [data] } }))
    })
  }
  
  render() {
    const { children, operator, status, onUpdate } = this.props;

    let list_items = children.map((c, i) => <div key={i} className="wizard-resource-status-item" data-satisfied={c.satisfied}>
      <RequirementStatusComponent
        status={c}
        onUpdate={withValU(onUpdate, onUp => (data: ResourceRequirements | null) => {
          if (data == null)
            if (children.length === 1)
              onUp(null)
            else 
              onUp(update(status.original, { [operator]: { $splice: [[ i, 1 ]] } }));
          else {
            onUp(update(status.original, { [operator]: { [i]: { $set: data } } }));
          }
        })}
      />
    </div>);

    if (operator == "Or") {
      list_items = interleave(list_items, () => <ListGroup.Item key="or" className="wizard-resource-status-or">
        <span className="text-muted">OR</span>
      </ListGroup.Item>);
    }

    return <Card className="wizard-resource-status" bg={status.satisfied ? "success" : "dark"}>
      <ListGroup variant="flush">
        { list_items }
        {
          withValU(onUpdate, onUp => <ListGroup.Item className="text-muted">
            <a onClick={() => this.newEntry(onUp)}>
              <FontAwesomeIcon icon={faAdd} /> Add Requirement ({ operator.toUpperCase() })
            </a>
          </ListGroup.Item>)
        }
      </ListGroup>
    </Card>
  }
}

type ResourceQuotaComponentProps = {
  quota: MappedResourceQuota,
  status: ResourceRequirementStatus,
  onUpdate?: (data: ResourceQuota | null) => void
};

class ResourceQuotaComponent extends React.PureComponent<ResourceQuotaComponentProps> {
  edit = (onUpdate: NonNullable<ResourceQuotaComponentProps["onUpdate"]>) => {
    confirmModal("", {
      data: this.props.quota as ResourceQuota,
      title: "Edit Quota",
      okText: "Confirm",
      renderInner: (data: ResourceQuota, onUpdate) => (
        <EditResourceQuota value={data} onUpdate={s => onUpdate(update(data, s))} />
      )
    }).then(data => {
      onUpdate(data)
    })
  }

  render() {
    const { quota, status, onUpdate } = this.props;
    const { ready, satisfied } = status;

    return <ListGroup.Item className="wizard-resource-status-quota" data-satisfied={satisfied}>
      <Row>
        <Col>
          { quota.template.fta ? "[FTA]" : undefined } <ResourceRoleLabel role={quota.template.role} />
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
        {
          withVal(onUpdate, onUp => <Col md="auto">
            <a className="text-muted" onClick={() => this.edit(onUp)}> <FontAwesomeIcon icon={faPencil} /> </a>
            &nbsp;&nbsp;
            <a className="text-danger" onClick={() => onUp(null)}> <FontAwesomeIcon icon={faTrash} /> </a>
          </Col>)
        }
      </Row>
    </ListGroup.Item>
  }
}

type EditResourceRequirementProps = {
  value: ResourceRequirements,
  onUpdate: (rr: Spec<ResourceRequirements>) => void
};

class EditResourceRequirement extends React.Component<EditResourceRequirementProps> {
  renderList = (operator: "And" | "Or", elements: ResourceRequirements[]) => {
    return <React.Fragment>
      {
        elements.map((el, i) => (
          <Card className="border-light my-3">
            <Card.Body>
              <EditResourceRequirement
                value={el}
                onUpdate={v => this.props.onUpdate({ [operator]: { [i]: { $set: update(el, v) } } })}
              />
              <Row className="mt-2">
                <Col className="text-end">
                  <Button
                    variant="danger"
                    size="sm"
                    disabled={elements.length === 1}
                    onClick={() => this.props.onUpdate({ [operator]: { $splice: [[i, 1]] } })}
                  >
                    <FontAwesomeIcon icon={faTrash} /> Delete
                  </Button>
                </Col>
              </Row>
            </Card.Body>
          </Card>
        ))
      }
      <a className="mt-2 text-muted" onClick={() => this.props.onUpdate({ [operator]: { $push: [ {Quota: NEW_DEFAULTS.Quota} ] } })}>
        <FontAwesomeIcon icon={faAdd} /> Add Requirement ({ operator.toUpperCase() })
      </a>
    </React.Fragment>
  }
  
  render() {
    const { value, onUpdate } = this.props;
    return <React.Fragment>
      <Row>
        <Col>
          <EnumToggleGroup
            name="resource_type"
            value={Object.keys(value)[0] as any}
            values={["And", "Or", "Quota"]}
            variant="secondary"
            onChange={(type: VariantKeys<ResourceRequirements>) => {
              onUpdate({ $set: { [type]: NEW_DEFAULTS[type] } } as Spec<ResourceRequirements>)
            }}
          />
        </Col>
      </Row>
      {
        "Quota" in value ? 
          <EditResourceQuota value={value.Quota} onUpdate={q => onUpdate({ Quota: q })} />
          : "And" in value ? this.renderList("And", value.And)
          : this.renderList("Or", value.Or) 
      }
    </React.Fragment>
  }
}

type EditResourceQuotaProps = {
  value: ResourceQuota,
  onUpdate: (q: Spec<ResourceQuota>) => void,
};

class EditResourceQuota extends React.Component<EditResourceQuotaProps> {
  render() {
    const { value, onUpdate } = this.props;
    const { template, min, max } = value;

    return <React.Fragment>
      <Row className="my-3">
        <Col>
          <Form.Select value={JSON.stringify(template.role)} onChange={(v) => onUpdate({ template: { role: { $set: JSON.parse(v.target.value) as ResourceRole } } })}>
            {
              ROLES.map((role, i) => <option key={i} value={JSON.stringify(role)}> <ResourceRoleLabel role={role} /> </option>)
            }
          </Form.Select>
        </Col>
        <Col md={3} className="text-end">
          <Button variant={template.fta ? "gold" : "secondary"} onClick={() => onUpdate({ template: { fta: { $set: !(template.fta || false) } } })}>
            { template.fta ? <React.Fragment> <FontAwesomeIcon icon={faCrown} /> &nbsp; FTA </React.Fragment> : "Not FTA"  }
          </Button>
        </Col>
      </Row>
      <Row>
        <Col md="auto"> <Form.Label className="text-muted"> # Resources:  </Form.Label> </Col>
        <Col md={2}>
          <BufferedFormControl
            instant
            type="number"
            min={0}
            max={max || undefined}
            value={min}
            onUpdate={v => onUpdate({ min: { $set: Number(v) } })}
          />
        </Col>
        <Col md="auto"> - </Col>
        <Col md={2}>
          <BufferedFormControl
            instant
            type="number"
            min={min}
            value={max || ""}
            onUpdate={v => onUpdate({ max: { $set: nullIfEmpty(String(v)) ? Number(v) : undefined } })}
          />
        </Col>
        <Col className="text-end">
          <Button
            variant={template.ready ? "good" : "secondary"} 
            onClick={() => onUpdate({ template: { ready: { $set: !(template.ready || false) } } })}
          >
            <FontAwesomeIcon icon={template.ready ? faCheck : faTimes} /> &nbsp; { template.ready ? "Req. Ready" : "No Ready" }
          </Button>
        </Col>
      </Row>
    </React.Fragment>
  }
}