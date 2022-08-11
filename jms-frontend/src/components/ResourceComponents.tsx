import React from "react";
import { Accordion } from "react-bootstrap";
import { capitalise } from "support/strings";
import { role2id, role2string } from "support/ws-additional";
import { MappedResourceQuota, ResourceRequirementStatus, ResourceRole } from "ws-schema";
import SimpleTooltip from "./elements/SimpleTooltip";
import FieldPosSelector, { FieldResource } from "./FieldPosSelector";

export class ResourceRoleLabel extends React.PureComponent<{ role: ResourceRole, fta?: boolean }> {
  renderRole() {
    return role2string(this.props.role)
  }

  render() {
    return this.props.fta ? <React.Fragment><strong>[FTA]&nbsp;</strong> {this.renderRole()} </React.Fragment> : this.renderRole();
  }
}

function getAllQuotas(status: ResourceRequirementStatus): MappedResourceQuota[] {
  let all: MappedResourceQuota[] = [];
  if ("Quota" in status.element) {
    all.push(status.element.Quota);
  } else if ("And" in status.element) {
    all = all.concat(status.element.And.flatMap(el => getAllQuotas(el)))
  } else if ("Or" in status.element) {
    all = all.concat(status.element.Or.flatMap(el => getAllQuotas(el)))
  }
  return all;
}

type ResourceRequirementMinimapProps = {
  status: ResourceRequirementStatus
};

export class ResourceRequirementMinimap extends React.PureComponent<ResourceRequirementMinimapProps> {
  render() {
    const { status } = this.props;

    const quotas = getAllQuotas(status);

    return <FieldPosSelector>
      {
        quotas.map(q => <FieldResource role={q.template.role} fta={q.template.fta}>
          <SimpleTooltip tip={ <ResourceRoleLabel fta={q.template.fta} role={q.template.role} /> }>
            <div className="resource-indicator" data-role={role2id(q.template.role)} data-fta={q.template.fta} data-satisfied={q.satisfied} data-ready={q.ready}>
              { q.resource_ids.length }
            </div>
          </SimpleTooltip>
        </FieldResource>)
      }
    </FieldPosSelector>
  }
}

export class ResourceRequirementMinimapAccordion extends React.PureComponent<{ defaultOpen?: boolean } & ResourceRequirementMinimapProps> {
  render() {
    const { defaultOpen, ...props } = this.props;

    return <Accordion defaultActiveKey={defaultOpen ? "0" : undefined}>
      <Accordion.Item eventKey="0">
        <Accordion.Header className="resource-accordion-header" data-ready={props.status.ready}> Resource Allocation Map </Accordion.Header>
        <Accordion.Body>
          <ResourceRequirementMinimap {...props} />
        </Accordion.Body>
      </Accordion.Item>
    </Accordion>
  }
}