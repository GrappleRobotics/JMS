import "./BufferedFormControl.scss";
import React from "react";
import { FormControl, FormControlProps } from "react-bootstrap";
import { Combine } from "../support/util";
import { nullIfEmpty } from "../support/strings";

export type BufferedProps = Combine<{
  autofocus?: boolean,
  auto?: boolean,
  autoMillis?: number,
  instant?: boolean,
  enter?: boolean,
  value: number | string,
  onUpdate?: (val: number | string) => void,
  className?: string,
  resetOnUpdate?: boolean,
  updateOnDefocus?: boolean,
  onEnter?: (val: number | string) => void,
}, Combine<FormControlProps, React.InputHTMLAttributes<HTMLInputElement>>>;

type BufferedState = {
  value: number | string
};

export default class BufferedFormControl extends React.Component<BufferedProps, BufferedState> {
  static defaultProps = {
    autofocus: false,
    auto: false,
    autoMillis: 250,
    instant: false,
    enter: true,
    updateOnDefocus: true,
  };

  private controlRef = React.createRef<HTMLInputElement>();
  private timer: any = undefined;

  readonly state = {
    value: this.props.value
  }

  componentDidUpdate(prevProps: BufferedProps) {
    if (prevProps.value !== this.props.value) {
      // @ts-ignore
      this.setState({ value: this.props.value || "" });    // Have to || "" otherwise the most controls won't update
    }
  }

  componentDidMount() {
    if (this.props.autofocus) {
      this.controlRef.current?.focus();
    }
  }

  valuesDiffer = () => {
    let oldV = String(this.props.value || "");
    let newV = String(this.state.value || "");

    return oldV !== newV;
  }

  triggerUpdate = () => {
    this.props.onUpdate?.(this.state.value);
    if (this.props.resetOnUpdate)
      this.setState({ value: this.props.value });
  }

  changed = (event: any) => {
    if (this.props.auto) {
      clearTimeout(this.timer);
      this.timer = setTimeout(() => this.triggerUpdate(), this.props.autoMillis || 250);
    }

    let value = event.target.value;

    if (this.props.type === "number" && nullIfEmpty(String(value)) !== null && Number(value) !== null) {
      let n = Number(value);
      value = Math.min(Math.max(n, Number(this.props.min || Number.NEGATIVE_INFINITY)), Number(this.props.max || Number.POSITIVE_INFINITY));
    }

    // @ts-ignore
    this.setState({ value: value }, this.props.instant ? this.triggerUpdate : undefined);
  }

  onKeyDown = (e : {key: string}) => {
    if (this.props.enter && e.key === 'Enter') {
      this.triggerUpdate();
      this.props.onEnter?.(this.state.value);
    }
  }

  focusInput = () => {
    // @ts-ignore
    this.controlRef.current?.focus();
  }

  render() {
    let { className, onUpdate, onEnter, autofocus, auto, autoMillis, instant, enter, resetOnUpdate, updateOnDefocus, ...props } = this.props;

    return <FormControl
      {...props}
      ref={this.controlRef}
      className={className}
      data-buffer-diff={this.valuesDiffer()}
      onChange={this.changed}
      onKeyDown={this.onKeyDown}
      value={this.state.value}
      onBlur={e => {
        if (updateOnDefocus)
          this.triggerUpdate();
      }}
    />;
  }
}