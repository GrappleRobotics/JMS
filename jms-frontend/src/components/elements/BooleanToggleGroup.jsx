import { faCheck, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { ToggleButton, ToggleButtonGroup } from "react-bootstrap";

export default class BooleanToggleGroup extends React.PureComponent {
  render() {
    let { name, value, onChange, ...props } = this.props;

    return <ToggleButtonGroup className="bool-toggle" name={name} value={value ? 1 : 0} onChange={v => onChange(v > 0)} type="radio" { ...props }>
      <ToggleButton variant={null} className="true" value={1}>
        <FontAwesomeIcon icon={faCheck} />
      </ToggleButton>

      <ToggleButton variant={null} className="false" value={0}>
        <FontAwesomeIcon icon={faTimes} />
      </ToggleButton>
    </ToggleButtonGroup>
  }
}