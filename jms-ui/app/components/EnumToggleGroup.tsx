import React from "react";
import { Button, ButtonGroup, ButtonGroupProps } from "react-bootstrap";
import { Combine } from "../support/util";

type EnumToggleGroupProps<T> = Combine<{
  name: string,
  values: T[],
  names?: string[],
  disabled?: boolean,
  value: T,
  onChange: (v: T) => void,
  variant: string,
  variantActive: string
}, Omit<ButtonGroupProps, "type">>;

export default class EnumToggleGroup<T> extends React.PureComponent<EnumToggleGroupProps<T>> {
  static defaultProps = {
    disabled: false,
    variant: "outline-primary",
    variantActive: "primary"
  }
  
  render() {
    let { name, names, disabled, values, value, variant, variantActive, onChange, ...props } = this.props;

    return <ButtonGroup name={name} {...props}>
      {
        values.map((v, i) => <Button key={i} variant={value === v ? variantActive : variant} data-selected={value === v} disabled={disabled} onClick={() => onChange(v)}>
          { names ? names[i] : String(v) }
        </Button>)
      }
    </ButtonGroup>
  }
}