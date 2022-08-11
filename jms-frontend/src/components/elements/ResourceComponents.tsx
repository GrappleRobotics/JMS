import React from "react";
import { capitalise } from "support/strings";
import { ResourceRole } from "ws-schema";

export class ResourceRoleLabel extends React.PureComponent<{ role: ResourceRole, fta?: boolean }> {
  renderRole() {
    const { role } = this.props;

    if (typeof role === "object") {
      if ("RefereePanel" in role) {
        if (role.RefereePanel === "HeadReferee")
          return "Referee: Head Ref";
        else 
          return `Referee: ${capitalise(role.RefereePanel.Alliance[0])} ${capitalise(role.RefereePanel.Alliance[1])}`;
      } else if ("ScorerPanel" in role) {
        return `Scorer: ${role.ScorerPanel.goals}${role.ScorerPanel.height[0].toUpperCase()}`
      } else if ("TeamEStop" in role) {
        return `EStop: ${capitalise(role.TeamEStop.alliance)} ${role.TeamEStop.station}`
      }
      return JSON.stringify(role);
    } else {
      const ROLE_LABEL_MAP: { [k in typeof role]: string } = {
        "Unknown": "Other",
        "Any": "Any",
        "ScorekeeperPanel": "Scorekeeper",
        "MonitorPanel": "Monitor",
        "TimerPanel": "Timer",
        "AudienceDisplay": "Audience Display",
        "FieldElectronics": "Field Electronics",
      }

      return ROLE_LABEL_MAP[role];
    }
  }

  render() {
    return this.props.fta ? <React.Fragment><strong>[FTA]&nbsp;</strong> {this.renderRole()} </React.Fragment> : this.renderRole();
  }
}