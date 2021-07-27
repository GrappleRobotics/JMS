import React from "react";
import { ToggleButton, ToggleButtonGroup } from "react-bootstrap";

export default class EnumToggleGroup extends React.Component {
  render() {
    let { name, variant, outline, values, value, ...props } = this.props;

    variant = variant || "primary";
    let variant_active = variant;
    if (outline)
      variant = "outline-" + variant;

    return <ToggleButtonGroup className="enum-toggle" name={name} type="radio" value={value} {...props}>
      {
        values.map(v => <ToggleButton variant={ value == v ? variant_active : variant } value={v}>
          { v }
        </ToggleButton>)
      }
    </ToggleButtonGroup>
  }
}