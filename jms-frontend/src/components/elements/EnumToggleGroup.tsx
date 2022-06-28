import React from "react";
import { ToggleButton, ToggleButtonGroup } from "react-bootstrap";
import { Combine } from "support/util";

type EnumToggleGroupProps = Combine<{
  name: string,
  values: string[],
  disabled: boolean
}, React.ComponentProps<ToggleButtonGroup<string>>>;

export default class EnumToggleGroup extends React.PureComponent<EnumToggleGroupProps> {
  static defaultProps = {
    disabled: false
  }
  
  render() {
    let { className, name, disabled, values, value, ...props } = this.props;

    return <ToggleButtonGroup className={`enum-toggle ${className}`} name={name} type="radio" value={value} {...props}>
      {
        values.map(v => <ToggleButton disabled={disabled} data-active={ value == v } value={v}>
          { v }
        </ToggleButton>)
      }
    </ToggleButtonGroup>
  }
}