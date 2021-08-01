import React from "react";

export default class Buffered extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      value: this.props.value
    }

    this.timer = null;
    this.controlRef = React.createRef();
  }

  componentDidUpdate(prevProps) {
    if (prevProps.value !== this.props.value) {
      this.setState({ value: this.props.value || "" });    // Have to || "" otherwise the most controls won't update
    }
  }

  componentDidMount() {
    if (this.props.autofocus) {
      this.controlRef.current.focus();
    }
  }

  valuesDiffer = () => {
    let oldV = String(this.props.value || "");
    let newV = String(this.state.value || "");

    return oldV !== newV;
  }

  triggerUpdate = () => {
    this.props.onUpdate(this.state.value);
  }

  changed = (v) => {
    if (this.props.auto) {
      clearTimeout(this.timer);
      this.timer = setTimeout(() => this.triggerUpdate(), this.props.autoMillis || 250);
    }
    this.setState({ value: (this.props.eventMap || (e => e.target.value))(v) }, this.props.instant ? this.triggerUpdate : null);
  }

  onKeyDown = (e) => {
    if (!!!this.props.noEnter && e.key === 'Enter') {
      this.triggerUpdate();
    }
  }

  focusInput = () => {
    this.controlRef.current.focus();
  }

  render() {
    let { auto, instant, className, onUpdate, value, children, changeKey, valueKey, eventMap, ...props } = this.props;

    return React.cloneElement(children, {
      ref: this.controlRef,
      className: (className || "") + " " + (this.valuesDiffer() ? "buffer-diff" : "buffer-same"),
      [valueKey || 'value']: this.state.value,
      [changeKey || 'onChange']: this.changed,
      onKeyDown: this.onKeyDown,
      ...props
    });
  }
}