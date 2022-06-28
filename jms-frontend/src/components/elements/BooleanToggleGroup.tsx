import { faCheck, faTimes } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import React from "react";
import { ToggleButton, ToggleButtonGroup } from "react-bootstrap";
import { Combine } from "support/util";

// type ToggleGroupProps = {
//   name: string,
//   value: boolean,
//   onChange: (v: boolean) => void,
// } & Omit<React.ComponentProps<ToggleButtonGroup<boolean>>, "value" | "onChange">;

type ToggleGroupProps = Combine<{
  name: string,
  value: boolean,
  onChange: (v: boolean) => void,
}, React.ComponentProps<ToggleButtonGroup<boolean>>>;

export default class BooleanToggleGroup extends React.PureComponent<ToggleGroupProps> {
  render() {
    let { className, name, value, onChange, ...props } = this.props;

    return <ToggleButtonGroup className={`bool-toggle ${className}`} name={name} value={value ? 1 : 0} onChange={v => onChange(v > 0)} type="radio" { ...props }>
      <ToggleButton variant={undefined} className="true" value={1}>
        <FontAwesomeIcon icon={faCheck} />
      </ToggleButton>

      <ToggleButton variant={undefined} className="false" value={0}>
        <FontAwesomeIcon icon={faTimes} />
      </ToggleButton>
    </ToggleButtonGroup>
  }
}